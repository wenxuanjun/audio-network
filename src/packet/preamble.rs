use crate::{modem::Modem, number::FP};
use std::marker::PhantomData;

#[cfg(feature = "cable_link")]
pub const PREAMBLE_LENGTH: usize = 240;
#[cfg(not(feature = "cable_link"))]
pub const PREAMBLE_LENGTH: usize = 480;

pub struct PreambleSequence<M> {
    modem: PhantomData<M>,
}

impl<M: Modem> PreambleSequence<M> {
    pub fn new(sample_rate: usize) -> Vec<FP> {
        let (freq_min, freq_max) = <M as Modem>::PREAMBLE_FREQUENCY_RANGE;

        let frequency_diff = FP::from(freq_max) - FP::from(freq_min);
        let preamble_center = FP::from(PREAMBLE_LENGTH) / FP::from(2.0);

        let get_frequency = |index: usize| {
            if index < FP::into::<usize>(preamble_center) {
                let ratio = FP::from(index) / preamble_center;
                FP::from(freq_min) + frequency_diff * ratio
            } else {
                let ratio = (FP::from(index) - preamble_center) / preamble_center;
                FP::from(freq_max) - frequency_diff * ratio
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
