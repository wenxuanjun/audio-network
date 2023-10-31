use super::{BitByteConverter, Modem};
use crate::number::FP;
use rustfft::FftDirection::{Forward, Inverse};
use rustfft::{algorithm::Radix4, num_complex::Complex, Fft};

const BIT_PER_SYMBOL: usize = 4;
const DATA_SYMBOL_PER_PACKET: usize = 48;

const DATA_SAMPLES: usize = 128;
const CYCLIC_PREFIX_SAMPLES: usize = 12;
const SAMPLES_PER_SYMBOL: usize = DATA_SAMPLES + CYCLIC_PREFIX_SAMPLES;

const FFT_ENERGY_ZOOM: f32 = 1.0 / 4.0;
const START_SUB_CARRIER_INDEX: usize = 18;

const SYMBOL_PER_PACKET: usize = DATA_SYMBOL_PER_PACKET + 1;
const PACKET_SAMPLES: usize = SYMBOL_PER_PACKET * SAMPLES_PER_SYMBOL;
const PACKET_DATA_BYTES: usize = BIT_PER_SYMBOL * DATA_SYMBOL_PER_PACKET / 8;

pub struct Ofdm {
    standard_phase: [Complex<f32>; 2],
    ffts: [Radix4<f32>; 2],
}

impl Modem for Ofdm {
    const PREFERED_PAYLOAD_BYTES: usize = 48;

    fn new(_: usize) -> Self {
        let ffts = [
            Radix4::new(DATA_SAMPLES, Forward),
            Radix4::new(DATA_SAMPLES, Inverse),
        ];

        let standard_phase = [
            Complex::new(FFT_ENERGY_ZOOM, 0.0),
            Complex::new(-FFT_ENERGY_ZOOM, 0.0),
        ];

        Self {
            standard_phase,
            ffts,
        }
    }

    fn modulate(&self, bytes: &Vec<u8>) -> Vec<FP> {
        assert!(
            bytes.len() % PACKET_DATA_BYTES == 0,
            "Bad data length: {}, can only modulate N * {} bytes per time!",
            bytes.len(),
            PACKET_DATA_BYTES
        );

        bytes
            .chunks(PACKET_DATA_BYTES)
            .flat_map(|chunk| self.encode_packet(chunk))
            .collect()
    }

    fn demodulate(&self, samples: &Vec<FP>) -> Vec<u8> {
        assert!(
            samples.len() % PACKET_SAMPLES == 0,
            "Bad data length: {}, can only demodulate N * {} samples per time!",
            samples.len(),
            PACKET_SAMPLES
        );

        let data_bytes = samples
            .chunks(PACKET_SAMPLES)
            .flat_map(|chunk| self.decode_packet(chunk))
            .collect::<Vec<_>>();

        BitByteConverter::bits_to_bytes(&data_bytes)
    }
}

impl Ofdm {
    fn encode_packet(&self, chunk: &[u8]) -> Vec<FP> {
        let bits = {
            let train_empty_bits = [0u8; BIT_PER_SYMBOL];
            let data_bits = BitByteConverter::bytes_to_bits(chunk);

            train_empty_bits
                .iter()
                .chain(data_bits.iter())
                .cloned()
                .collect::<Vec<_>>()
        };

        bits.chunks(BIT_PER_SYMBOL)
            .flat_map(|chunk| {
                let mut buffer = vec![Complex::default(); DATA_SAMPLES];

                buffer
                    .iter_mut()
                    .skip(START_SUB_CARRIER_INDEX)
                    .zip(chunk.iter())
                    .for_each(|(buffer, bit)| *buffer = self.standard_phase[*bit as usize]);

                self.ffts[1].process(&mut buffer);

                buffer
                    .iter()
                    .map(|x| FP::from(x.re))
                    .skip(DATA_SAMPLES - CYCLIC_PREFIX_SAMPLES)
                    .chain(buffer.iter().map(|x| FP::from(x.re)))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }

    fn decode_packet(&self, chunk: &[FP]) -> Vec<u8> {
        let (train_samples, data_samples) = chunk.split_at(SAMPLES_PER_SYMBOL);

        let train_args = {
            let mut buffer = train_samples[CYCLIC_PREFIX_SAMPLES..]
                .iter()
                .map(|x| Complex::new(FP::into(*x), 0.0))
                .collect::<Vec<_>>();

            self.ffts[0].process(&mut buffer);

            (0..BIT_PER_SYMBOL)
                .map(|index| buffer[START_SUB_CARRIER_INDEX + index].arg())
                .collect::<Vec<_>>()
        };

        data_samples
            .chunks(SAMPLES_PER_SYMBOL)
            .flat_map(|chunk| {
                let mut buffer = vec![Complex::default(); DATA_SAMPLES];

                buffer
                    .iter_mut()
                    .zip(chunk[CYCLIC_PREFIX_SAMPLES..].iter())
                    .for_each(|(x, y)| *x = Complex::new(FP::into(*y), 0.0));

                self.ffts[0].process(&mut buffer);

                (0..BIT_PER_SYMBOL)
                    .map(|index| {
                        let offset = Complex::exp(-Complex::new(0.0, 1.0) * train_args[index]);
                        ((buffer[START_SUB_CARRIER_INDEX + index] * offset).re < 0.0) as u8
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: usize = 48000;
    const TEST_SEQUENCE_BYTES: usize = 36;

    #[test]
    fn test_ofdm() {
        let data = (0..TEST_SEQUENCE_BYTES).map(|index| index as u8).collect();

        let ofdm = Ofdm::new(SAMPLE_RATE);

        let mut modulated = ofdm.modulate(&data);

        modulated
            .iter_mut()
            .for_each(|sample| *sample += FP::from(rand::random::<f32>()) / FP::from(2.0));

        let demodulated = ofdm.demodulate(&modulated);

        assert_eq!(data, demodulated);
    }
}
