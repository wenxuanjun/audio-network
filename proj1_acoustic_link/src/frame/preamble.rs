use crate::number::FP;

pub const PREAMBLE_LENGTH: usize = 480;
const PREAMBLE_FREQ_MIN: f32 = 3600.0;
const PREAMBLE_FREQ_MAX: f32 = 5200.0;

pub struct PreambleSequence;

impl PreambleSequence {
    pub fn new(sample_rate: usize) -> Vec<FP> {
        let preamble_center: FP = FP::from(PREAMBLE_LENGTH) / FP::from(2.0);
        let frequency_diff: FP = FP::from(PREAMBLE_FREQ_MAX) - FP::from(PREAMBLE_FREQ_MIN);

        let get_frequency = |index: usize| {
            if index < FP::into::<usize>(preamble_center) {
                let ratio = FP::from(index) / preamble_center;
                FP::from(PREAMBLE_FREQ_MIN) + frequency_diff * ratio
            } else {
                let ratio = (FP::from(index) - preamble_center) / preamble_center;
                FP::from(PREAMBLE_FREQ_MAX) - frequency_diff * ratio
            }
        };

        let mut integral: FP = FP::ZERO;
        let mut preamble_samples: Vec<FP> = Vec::with_capacity(PREAMBLE_LENGTH);

        for index in 0..PREAMBLE_LENGTH {
            integral += get_frequency(index) / FP::from(sample_rate);
            preamble_samples.push((integral * FP::from(2.0) * FP::PI).sin());
        }

        preamble_samples
    }
}

