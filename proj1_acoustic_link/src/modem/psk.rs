use super::{BitByteConverter, Modem};

pub struct PSK {
    sample_rate: usize,
    standard_frame: StandardFrame,
}

struct StandardFrame {
    zero_frame: Vec<f32>,
    one_frame: Vec<f32>,
}

impl Modem for PSK {
    const BIT_RATE: usize = 1000;
    const CARRIER_FREQUENCY: f32 = 2400.0;

    fn modulate(&self, bytes: &Vec<u8>) -> Vec<f32> {
        BitByteConverter::bytes_to_bits(bytes)
            .into_iter()
            .map(|bit| {
                let standard = &self.standard_frame;
                match bit {
                    0 => standard.zero_frame.clone(),
                    1 => standard.one_frame.clone(),
                    _ => panic!("Only 0 or 1 is valid bit!"),
                }
            })
            .flatten()
            .collect()
    }

    fn demodulate(&self, samples: &Vec<f32>) -> Vec<u8> {
        let bits = samples
            .chunks((self.sample_rate / PSK::BIT_RATE) as usize)
            .map(|frame| {
                let similarity = frame
                    .iter()
                    .zip(self.standard_frame.zero_frame.iter())
                    .map(|(a, b)| a * b)
                    .sum::<f32>();
                (similarity < 0.0) as u8
            })
            .collect::<Vec<_>>();

        BitByteConverter::bits_to_bytes(&bits)
    }
}

impl PSK {
    pub fn new(sample_rate: usize) -> Self {
        let sine_wave = |index| {
            (index as f32 / sample_rate as f32
                * 2.0
                * std::f32::consts::PI
                * PSK::CARRIER_FREQUENCY)
                .sin()
        };

        let frame_length = sample_rate / PSK::BIT_RATE;
        let zero_frame: Vec<_> = (0..frame_length).map(sine_wave).collect();
        let one_frame: Vec<_> = zero_frame.iter().map(|item| -item).collect();

        let standard_frame = StandardFrame {
            zero_frame,
            one_frame,
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
