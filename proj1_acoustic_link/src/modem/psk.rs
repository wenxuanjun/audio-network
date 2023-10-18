use super::Modem;
pub struct PSK {
    sample_rate: usize,
    standard_frame: StandardFrame,
}

struct StandardFrame {
    zero: Vec<f32>,
    one: Vec<f32>,
}

impl Modem for PSK {
    const BIT_RATE: usize = 1000;
    const CARRIER_FREQUENCY: f32 = 2400.0;

    fn modulate(&self, bits: &Vec<u8>) -> Vec<f32> {
        bits.into_iter()
            .map(|bit| {
                let standard = &self.standard_frame;
                match bit {
                    0 => standard.zero.clone(),
                    1 => standard.one.clone(),
                    _ => panic!("Only 0 or 1 is valid bit!"),
                }
            })
            .flatten()
            .collect()
    }

    fn demodulate(&self, samples: &Vec<f32>) -> Vec<u8> {
        let vector_product = |seq_a: &[f32], seq_b: &[f32]| -> f32 {
            seq_a.iter().zip(seq_b.iter()).map(|(a, b)| a * b).sum()
        };

        samples
            .chunks((self.sample_rate / PSK::BIT_RATE) as usize)
            .map(|frame| {
                let similarity = vector_product(frame, &self.standard_frame.zero);
                (similarity < 0.0) as u8
            })
            .collect()
    }
}

impl PSK {
    pub fn new(sample_rate: usize) -> Self {
        let standard_frame = {
            let sine_wave = |index| {
                let multiplier = {
                    let current_ratio = index as f32 / sample_rate as f32;
                    2.0 * std::f32::consts::PI * current_ratio
                };
                (multiplier * PSK::CARRIER_FREQUENCY).sin()
            };

            let frame_length = sample_rate / PSK::BIT_RATE;
            let zero_frame: Vec<f32> = (0..frame_length).map(sine_wave).collect();
            let one_frame: Vec<_> = zero_frame.iter().map(|item| -item).collect();

            StandardFrame { zero: zero_frame, one: one_frame }
        };

        Self {
            sample_rate,
            standard_frame,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: usize = 48000;
    const TEST_SEQUENCE_LENGTH: usize = 100;

    #[test]
    fn test_psk() {
        let psk = PSK::new(SAMPLE_RATE);

        let data = (0..TEST_SEQUENCE_LENGTH)
            .map(|_| rand::random::<u8>() % 2)
            .collect();

        let mut modulated = psk.modulate(&data);

        modulated
            .iter_mut()
            .for_each(|sample| *sample += rand::random::<f32>() / 2.0);

        let demodulated = psk.demodulate(&modulated);

        assert_eq!(data, demodulated);
    }
}
