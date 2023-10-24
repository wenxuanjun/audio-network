use super::{BitByteConverter, Modem};

const BIT_PER_SAMPLE: usize = 1;
const BAUD_RATE: usize = PSK::BIT_RATE / BIT_PER_SAMPLE;
const FRAME_VARIANCE: usize = 2usize.pow(BIT_PER_SAMPLE as u32);

pub struct PSK {
    sample_rate: usize,
    standard_frame: [Vec<f32>; FRAME_VARIANCE],
    gray_code: [Vec<u8>; FRAME_VARIANCE],
}

impl Modem for PSK {
    const BIT_RATE: usize = 1000;
    const CARRIER_FREQUENCY: f32 = 2400.0;

    fn modulate(&self, bytes: &Vec<u8>) -> Vec<f32> {
        BitByteConverter::bytes_to_bits(bytes)
            .chunks(BIT_PER_SAMPLE)
            .map(|chunk| {
                let index = self
                    .gray_code
                    .iter()
                    .enumerate()
                    .find(|(_, code)| code == &chunk)
                    .unwrap()
                    .0;

                self.standard_frame[index as usize].clone()
            })
            .flatten()
            .collect()
    }

    fn demodulate(&self, samples: &Vec<f32>) -> Vec<u8> {
        let frame_length = self.sample_rate / BAUD_RATE;

        let bits = samples
            .chunks(frame_length)
            .map(|frame| {
                let similarities = self
                    .standard_frame
                    .iter()
                    .map(|standard| {
                        frame
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
            .flatten()
            .collect::<Vec<_>>();

        BitByteConverter::bits_to_bytes(&bits)
    }
}

impl PSK {
    pub fn new(sample_rate: usize) -> Self {
        let sine_frame = |length, phase| {
            (0..length)
                .map(|index| {
                    (index as f32 / sample_rate as f32
                        * 2.0
                        * std::f32::consts::PI
                        * PSK::CARRIER_FREQUENCY
                        + phase as f32)
                        .sin()
                })
                .collect::<Vec<_>>()
        };

        let start_phase = if BIT_PER_SAMPLE == 1 {
            0.0
        } else {
            std::f32::consts::PI / FRAME_VARIANCE as f32
        };

        let standard_frame = (0..2usize.pow(BIT_PER_SAMPLE as u32))
            .map(|index| {
                let round = std::f32::consts::PI * 2.0;
                sine_frame(
                    sample_rate / BAUD_RATE,
                    start_phase + index as f32 * round / FRAME_VARIANCE as f32,
                )
            })
            .collect::<Vec<_>>();

        let gray_code = |bits| {
            let mut gray_code = vec![vec![0], vec![1]];

            for _ in 0..bits - 1 {
                let mut reflected = gray_code.clone();
                reflected.reverse();
                for code in &mut reflected {
                    code.insert(0, 1);
                }
                for code in &mut gray_code {
                    code.insert(0, 0);
                }
                gray_code.extend(reflected);
            }

            gray_code
        };

        Self {
            sample_rate,
            standard_frame: standard_frame.try_into().unwrap(),
            gray_code: gray_code(BIT_PER_SAMPLE).try_into().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: usize = 48000;
    const TEST_SEQUENCE_BYTES: usize = 100;

    #[test]
    fn test_psk() {
        let psk = PSK::new(SAMPLE_RATE);

        let data = (0..TEST_SEQUENCE_BYTES)
            .map(|_| rand::random::<u8>())
            .collect();

        let mut modulated = psk.modulate(&data);

        modulated
            .iter_mut()
            .for_each(|sample| *sample += rand::random::<f32>() / 2.0);

        let demodulated = psk.demodulate(&modulated);

        assert_eq!(data, demodulated);
    }
}
