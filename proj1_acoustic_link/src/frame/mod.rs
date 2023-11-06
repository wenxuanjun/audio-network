use slice_deque::SliceDeque;
use crate::number::FP;

mod preamble;
pub use preamble::{PreambleSequence, PREAMBLE_LENGTH};

cfg_if::cfg_if! {
    if #[cfg(feature = "cable_link")] {
        const DETECT_THRETSHOLD_MIN: f32 = 50.0;
        const DETECT_THRETSHOLD_RATIO: f32 = 5.0;
    } else {
        const DETECT_THRETSHOLD_MIN: f32 = 20.0;
        const DETECT_THRETSHOLD_RATIO: f32 = 5.0;
    }
}

#[derive(PartialEq)]
pub enum FrameDetectorState {
    Payload,
    MaybePayload,
    Waiting,
}

pub struct FrameDetector {
    preamble: Vec<FP>,
    detect_buffer: SliceDeque<FP>,
    payload_buffer: Vec<FP>,
    current_state: FrameDetectorState,
    correlation_buffer: SliceDeque<FP>,
}

impl FrameDetector {
    pub fn new(preamble: Vec<FP>, payload_capacity: usize) -> Self {
        Self {
            preamble,
            detect_buffer: SliceDeque::with_capacity(PREAMBLE_LENGTH),
            payload_buffer: Vec::with_capacity(payload_capacity),
            current_state: FrameDetectorState::Waiting,
            correlation_buffer: SliceDeque::with_capacity(PREAMBLE_LENGTH),
        }
    }

    fn get_correlation(&self) -> FP {
        self.detect_buffer
            .iter()
            .zip(self.preamble.iter())
            .map(|(a, b)| *a * *b)
            .sum::<FP>()
    }

    pub fn update(&mut self, sample: FP) -> Option<&Vec<FP>> {
        if self.detect_buffer.len() == PREAMBLE_LENGTH {
            self.detect_buffer.pop_front();
        }
        self.detect_buffer.push_back(sample);

        if self.current_state == FrameDetectorState::MaybePayload {
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
                    .sum::<FP>()
                    / FP::from(PREAMBLE_LENGTH);

                if correlation > FP::from(DETECT_THRETSHOLD_MIN)
                    && correlation > average_correlation * FP::from(DETECT_THRETSHOLD_RATIO)
                {
                    self.current_state = FrameDetectorState::MaybePayload;
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
            FrameDetectorState::MaybePayload => unreachable!(),
        }
    }
}
