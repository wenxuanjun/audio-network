mod preamble;
pub use preamble::{PreambleSequence, PREAMBLE_LENGTH};

use crate::modem::{Modem, PSK};
use slice_deque::SliceDeque;

pub const PAYLOAD_LENGTH: usize = 100;
const DETECT_THRETSHOLD_MIN: f32 = 10.0;
const DETECT_THRETSHOLD_RATIO: f32 = 4.5;

#[derive(PartialEq)]
pub enum FrameDetectorState {
    Payload,
    MayBePayload,
    Waiting,
}

pub struct FrameDetector {
    preamble: Vec<f32>,
    detect_buffer: SliceDeque<f32>,
    payload_buffer: Vec<f32>,
    current_state: FrameDetectorState,
    correlation_buffer: SliceDeque<f32>,
}

impl FrameDetector {
    pub fn new(sample_rate: usize) -> Self {
        let payload_capacity = sample_rate / PSK::BIT_RATE * PAYLOAD_LENGTH;

        Self {
            preamble: PreambleSequence::new(sample_rate),
            detect_buffer: SliceDeque::with_capacity(PREAMBLE_LENGTH),
            payload_buffer: Vec::with_capacity(payload_capacity),
            current_state: FrameDetectorState::Waiting,
            correlation_buffer: SliceDeque::with_capacity(PREAMBLE_LENGTH),
        }
    }

    fn get_correlation(&self) -> f32 {
        self.detect_buffer
            .iter()
            .zip(self.preamble.iter())
            .map(|(a, b)| a * b)
            .sum::<f32>()
    }

    pub fn update(&mut self, sample: f32) -> Option<&Vec<f32>> {
        if self.detect_buffer.len() == PREAMBLE_LENGTH {
            self.detect_buffer.pop_front();
        }
        self.detect_buffer.push_back(sample);

        if self.current_state == FrameDetectorState::MayBePayload {
            if self.get_correlation() > *self.correlation_buffer.back().unwrap() {
                self.current_state = FrameDetectorState::Waiting;
            } else {
                self.current_state = FrameDetectorState::Payload;
            }
        }

        match self.current_state {
            FrameDetectorState::Waiting => {
                let correlation = self.get_correlation();

                if self.correlation_buffer.len() == PREAMBLE_LENGTH {
                    self.correlation_buffer.pop_front();
                }
                self.correlation_buffer.push_back(correlation);

                let average_correlation = self
                    .correlation_buffer
                    .iter()
                    .map(|&x| x.abs())
                    .sum::<f32>()
                    / PREAMBLE_LENGTH as f32;

                if correlation > DETECT_THRETSHOLD_MIN
                    && correlation > average_correlation * DETECT_THRETSHOLD_RATIO
                {
                    self.current_state = FrameDetectorState::MayBePayload;
                    self.payload_buffer.clear();
                }

                None
            }
            FrameDetectorState::Payload => {
                self.payload_buffer.push(sample);

                if self.payload_buffer.len() == self.payload_buffer.capacity() {
                    self.current_state = FrameDetectorState::Waiting;
                    return Some(&self.payload_buffer);
                }

                None
            }
            FrameDetectorState::MayBePayload => unreachable!(),
        }
    }
}
