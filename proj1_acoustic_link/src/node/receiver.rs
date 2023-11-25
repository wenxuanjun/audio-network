use crossbeam_channel::{unbounded, Receiver as ChannelReceiver};
use jack::ProcessScope;

use crate::audio::{Audio, AudioPorts};
use crate::frame::{FrameDetector, PreambleSequence};
use crate::modem::Modem;
use crate::number::FP;

pub struct Receiver<M> {
    modem: M,
    sample_receiver: ChannelReceiver<f32>,
    pub recorded_data: Vec<f32>,
    frame_detector: FrameDetector,
}

impl<M> Receiver<M>
where
    M: Modem + Sync + Send + 'static,
{
    pub fn new(audio: &'static Audio) -> Self {
        let (sample_sender, sample_receiver) = unbounded();

        let capture_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            ports.capture.as_slice(&ps).iter().for_each(|&sample| {
                sample_sender.send(sample).unwrap();
            });
        };

        audio.register(Box::new(capture_callback));
        info!("Capture demodulated data registered!");

        let sample_rate = audio.sample_rate.get().unwrap();
        let modem = M::new(sample_rate);

        let payload_capacity = {
            let payload_bytes = M::PREFERED_PAYLOAD_BYTES;
            let empty_frame = modem.modulate(&vec![0; payload_bytes]);
            empty_frame.len()
        };

        let preamble = PreambleSequence::<M>::new(sample_rate);
        let frame_detector = FrameDetector::new(preamble, payload_capacity);

        let recorded_data = Vec::new();

        Self {
            modem,
            sample_receiver,
            recorded_data,
            frame_detector,
        }
    }

    pub fn recv(&mut self) -> Vec<u8> {
        loop {
            let sample = self.sample_receiver.recv().unwrap();
            self.recorded_data.push(sample);

            if let Some(frame) = self.frame_detector.update(FP::from(sample)) {
                let demodulated_frame = self.modem.demodulate(frame);
                debug!("Demodulated frame: {:?}", demodulated_frame);
                return demodulated_frame;
            }
        }
    }
}
