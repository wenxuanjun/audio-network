use jack::ProcessScope;
use std::sync::{Arc, Mutex};

use crate::audio::{Audio, AudioPorts};
use crate::frame::FrameDetector;
use crate::modem::Modem;

#[derive(Default)]
pub struct ReceiverOutput {
    pub recorded_data: Vec<f32>,
    pub demodulated_data: Vec<u8>,
}

pub struct Receiver;

impl Receiver {
    pub fn register(audio: &'static Audio) -> Arc<Mutex<ReceiverOutput>> {
        let sample_rate = audio.sample_rate.borrow().unwrap();

        let received_output = Arc::new(Mutex::new(ReceiverOutput::default()));
        let psk = crate::modem::PSK::new(sample_rate);
        let mut frame_detector = FrameDetector::new(sample_rate);

        let received_output_clone = received_output.clone();
        let capture_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            let mut received_output = received_output_clone.lock().unwrap();

            ports.capture.as_slice(&ps).iter().for_each(|&sample| {
                received_output.recorded_data.push(sample);

                if let Some(frame) = frame_detector.update(sample) {
                    let demodulated_frame = psk.demodulate(frame);
                    println!("Demodulated frame: {:?}", demodulated_frame);
                    received_output.demodulated_data.extend(demodulated_frame);
                }
            });
        };

        audio.register(Box::new(capture_callback));

        println!("Capture demodulated data registered!");

        received_output
    }
}
