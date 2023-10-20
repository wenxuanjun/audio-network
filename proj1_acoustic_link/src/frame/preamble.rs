use std::f32::consts::PI;

pub const PREAMBLE_LENGTH: usize = 480;
const PREAMBLE_FREQ_MIN: f32 = 3600.0;
const PREAMBLE_FREQ_MAX: f32 = 5200.0;

pub struct PreambleSequence;

impl PreambleSequence {
    pub fn new(sample_rate: usize) -> Vec<f32> {
        const PREAMBLE_CENTER: f32 = PREAMBLE_LENGTH as f32 / 2.0;
        const FREQUENCY_DIFF: f32 = PREAMBLE_FREQ_MAX - PREAMBLE_FREQ_MIN;

        let get_frequency = |index: usize| {
            if index < PREAMBLE_CENTER as usize {
                let ratio = index as f32 / PREAMBLE_CENTER;
                PREAMBLE_FREQ_MIN + FREQUENCY_DIFF* ratio
            } else {
                let ratio = (index as f32 - PREAMBLE_CENTER) / PREAMBLE_CENTER;
                PREAMBLE_FREQ_MAX - FREQUENCY_DIFF * ratio
            }
        };

        let mut integral: f32 = 0.0;
        let mut preamble_samples: Vec<f32> = Vec::with_capacity(PREAMBLE_LENGTH);

        for index in 0..PREAMBLE_LENGTH {
            integral += get_frequency(index) / sample_rate as f32;
            preamble_samples.push((integral * 2.0 * PI).sin());
        }

        preamble_samples
    }
}
