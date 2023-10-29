use jack::ProcessScope;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::audio::{Audio, AudioPorts};
use crate::number::FP;
use crate::frame::FrameDetector;
use crate::modem::Modem;

#[derive(Default)]
pub struct ReceiverOutput {
    pub recorded_data: Vec<f32>,
    pub demodulated_data: Vec<u8>,
}

pub struct Receiver<M> {
    modem: PhantomData<M>,
}

impl<M> Receiver<M>
where
    M: Modem + Sync + Send + 'static,
{
    pub fn register(audio: &'static Audio) -> Arc<Mutex<ReceiverOutput>> {
        let sample_rate = audio.sample_rate.borrow().unwrap();

        let modem = <M as Modem>::new(sample_rate);

        let payload_capacity = {
            let payload_bytes = <M as Modem>::PREFERED_PAYLOAD_BYTES;
            let empty_frame = modem.modulate(&vec![0; payload_bytes]);
            empty_frame.len()
        };

        let mut frame_detector = FrameDetector::new(sample_rate, payload_capacity);

        let received_output = Arc::new(Mutex::new(ReceiverOutput::default()));
        let received_output_clone = received_output.clone();

        let capture_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            let mut received_output = received_output_clone.lock().unwrap();

            ports.capture.as_slice(&ps).iter().for_each(|&sample| {
                received_output.recorded_data.push(sample);

                if let Some(frame) = frame_detector.update(FP::from(sample)) {
                    let demodulated_frame = modem.demodulate(frame);
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
