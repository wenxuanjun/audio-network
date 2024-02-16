use super::{BitByteConverter, Modem};
use crate::number::FP;

cfg_if::cfg_if! {
    if #[cfg(feature = "cable_link")] {
        const BIT_PER_SYMBOL: usize = 1;
        const BIT_RATE_MUL_RATIO: usize = 1250;
        const CARRIER_FREQUENCY: f32 = 1600.0;
    } else {
        const BIT_PER_SYMBOL: usize = 1;
        const BIT_RATE_MUL_RATIO: usize = 1250;
        const CARRIER_FREQUENCY: f32 = 1600.0;
    }
}

const SYMBOL_RATE: usize = BIT_RATE_MUL_RATIO * BIT_PER_SYMBOL / BIT_PER_SYMBOL;
const CHUNK_VARIANCE: usize = 2usize.pow(BIT_PER_SYMBOL as u32);

pub struct Psk {
    sample_rate: usize,
    standard_chunk: [Vec<FP>; CHUNK_VARIANCE],
    gray_code: [Vec<u8>; CHUNK_VARIANCE],
}

impl Modem for Psk {
    cfg_if::cfg_if! {
        if #[cfg(feature = "cable_link")] {
            const PREFERED_PAYLOAD_BYTES: usize = 16;
            const PREAMBLE_FREQUENCY_RANGE: (f32, f32) = (900.0, 3000.0);
        } else {
            const PREFERED_PAYLOAD_BYTES: usize = 16;
            const PREAMBLE_FREQUENCY_RANGE: (f32, f32) = (900.0, 3000.0);
        }
    }
    const MIN_MODULATE_BYTES: usize = BIT_PER_SYMBOL;

    fn new(sample_rate: usize) -> Self {
        let gray_code = Self::gray_code(BIT_PER_SYMBOL);
        let standard_chunk = Self::standard_chunk(sample_rate);

        Self {
            sample_rate,
            standard_chunk: standard_chunk.try_into().unwrap(),
            gray_code: gray_code.try_into().unwrap(),
        }
    }

    fn modulate(&self, bytes: &[u8]) -> Vec<FP> {
        BitByteConverter::bytes_to_bits(bytes)
            .chunks(BIT_PER_SYMBOL)
            .flat_map(|chunk| {
                let index = self
                    .gray_code
                    .iter()
                    .enumerate()
                    .find(|(_, code)| code == &chunk)
                    .unwrap()
                    .0;

                self.standard_chunk[index as usize].clone()
            })
            .collect()
    }

    fn demodulate(&self, samples: &[FP]) -> Vec<u8> {
        let chunk_length = self.sample_rate / SYMBOL_RATE;

        let bits = samples
            .chunks(chunk_length)
            .flat_map(|chunk| {
                let similarities = self
                    .standard_chunk
                    .iter()
                    .map(|standard| {
                        chunk
                            .iter()
                            .zip(standard.iter())
                            .map(|(a, b)| *a * *b)
                            .sum::<FP>()
                    })
                    .collect::<Vec<_>>();

                let max_index = similarities
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .unwrap()
                    .0;

                self.gray_code[max_index as usize]
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        BitByteConverter::bits_to_bytes(&bits)
    }
}

impl Psk {
    fn gray_code(bits: usize) -> Vec<Vec<u8>> {
        let mut gray_code = vec![vec![0], vec![1]];

        (0..bits - 1).for_each(|_| {
            let mut reflected = gray_code.clone();
            reflected.reverse();
            reflected.iter_mut().for_each(|code| code.insert(0, 1));
            gray_code.iter_mut().for_each(|code| code.insert(0, 0));
            gray_code.extend(reflected);
        });

        gray_code
    }

    fn standard_chunk(sample_rate: usize) -> Vec<Vec<FP>> {
        let sine_chunk = |length, phase| {
            (0..length)
                .map(|index| {
                    let result: FP = FP::from(index)
                        / FP::from(sample_rate)
                        * FP::from(2.0)
                        * FP::PI
                        * FP::from(CARRIER_FREQUENCY)
                        + phase;
                    result.sin()
                })
                .collect::<Vec<_>>()
        };

        let start_phase = if BIT_PER_SYMBOL == 1 {
            FP::ZERO
        } else {
            FP::PI / FP::from(CHUNK_VARIANCE)
        };

        (0..2usize.pow(BIT_PER_SYMBOL as u32))
            .map(|index| {
                let round = FP::PI * FP::from(2.0);
                let phase_slice = round / FP::from(CHUNK_VARIANCE);
                
                sine_chunk(
                    sample_rate / SYMBOL_RATE,
                    start_phase + FP::from(index) * phase_slice,
                )
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: usize = 48000;
    const TEST_SEQUENCE_BYTES: usize = 1;

    #[test]
    fn test_psk() {
        let data = (0..TEST_SEQUENCE_BYTES)
            .map(|_| rand::random::<u8>())
            .collect::<Vec<_>>();

        let psk = Psk::new(SAMPLE_RATE);

        let mut modulated = psk.modulate(&data);

        modulated
            .iter_mut()
            .for_each(|sample| *sample += FP::from(rand::random::<f32>()) / FP::from(2.0));

        let demodulated = psk.demodulate(&modulated);

        assert_eq!(data, demodulated);
    }
}
