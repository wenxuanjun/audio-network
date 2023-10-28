use super::{BitByteConverter, Modem};

const BIT_PER_SYMBOL: usize = 1;
const SYMBOL_RATE: usize = Psk::BIT_RATE / BIT_PER_SYMBOL;
const CHUNK_VARIANCE: usize = 2usize.pow(BIT_PER_SYMBOL as u32);

pub struct Psk {
    sample_rate: usize,
    standard_chunk: [Vec<f32>; CHUNK_VARIANCE],
    gray_code: [Vec<u8>; CHUNK_VARIANCE],
}

impl Modem for Psk {
    const PREFERED_PAYLOAD_BYTES: usize = 16;

    fn new(sample_rate: usize) -> Self {
        let sine_chunk = |length, phase| {
            (0..length)
                .map(|index| {
                    (index as f32 / sample_rate as f32
                        * 2.0
                        * std::f32::consts::PI
                        * Psk::CARRIER_FREQUENCY
                        + phase as f32)
                        .sin()
                })
                .collect::<Vec<_>>()
        };

        let start_phase = if BIT_PER_SYMBOL == 1 {
            0.0
        } else {
            std::f32::consts::PI / CHUNK_VARIANCE as f32
        };

        let standard_chunk = (0..2usize.pow(BIT_PER_SYMBOL as u32))
            .map(|index| {
                let round = std::f32::consts::PI * 2.0;
                sine_chunk(
                    sample_rate / SYMBOL_RATE,
                    start_phase + index as f32 * round / CHUNK_VARIANCE as f32,
                )
            })
            .collect::<Vec<_>>();

        let gray_code = |bits| {
            let mut gray_code = vec![vec![0], vec![1]];

            (0..bits - 1).for_each(|_| {
                let mut reflected = gray_code.clone();
                reflected.reverse();
                reflected.iter_mut().for_each(|code| code.insert(0, 1));
                gray_code.iter_mut().for_each(|code| code.insert(0, 0));
                gray_code.extend(reflected);
            });

            gray_code
        };

        Self {
            sample_rate,
            standard_chunk: standard_chunk.try_into().unwrap(),
            gray_code: gray_code(BIT_PER_SYMBOL).try_into().unwrap(),
        }
    }

    fn modulate(&self, bytes: &Vec<u8>) -> Vec<f32> {
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

    fn demodulate(&self, samples: &Vec<f32>) -> Vec<u8> {
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
                            .map(|(a, b)| a * b)
                            .sum::<f32>()
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
    pub const BIT_RATE: usize = 1000;
    const CARRIER_FREQUENCY: f32 = 2400.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: usize = 48000;
    const TEST_SEQUENCE_BYTES: usize = 100;

    #[test]
    fn test_psk() {
        let data = (0..TEST_SEQUENCE_BYTES)
            .map(|_| rand::random::<u8>())
            .collect();

        let psk = Psk::new(SAMPLE_RATE);

        let mut modulated = psk.modulate(&data);

        modulated
            .iter_mut()
            .for_each(|sample| *sample += rand::random::<f32>() / 2.0);

        let demodulated = psk.demodulate(&modulated);

        assert_eq!(data, demodulated);
    }
}
