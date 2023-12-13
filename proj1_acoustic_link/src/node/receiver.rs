use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Receiver as ChannelReceiver};
use jack::ProcessScope;

use crate::audio::{Audio, AudioPorts};
use crate::frame::{FrameDetector, PreambleSequence};
use crate::modem::Modem;
use crate::number::FP;

const AVG_POWER_REFRESH_FACTOR: f32 = 0.85;

pub struct Receiver<M> {
    modem: M,
    sample_receiver: ChannelReceiver<f32>,
    frame_detector: Arc<Mutex<FrameDetector>>,
    pub recorded_data: Arc<Mutex<Vec<f32>>>,
    pub average_power: Arc<Mutex<f32>>,
}

impl<M> Receiver<M>
where
    M: Modem + Sync + Send + 'static,
{
    pub fn new(audio: &'static Audio) -> Self {
        let sample_rate = audio.sample_rate.get().unwrap();
        let modem = <M as Modem>::new(sample_rate);

        let payload_capacity = {
            let payload_bytes = M::PREFERED_PAYLOAD_BYTES;
            let empty_frame = modem.modulate(&vec![0; payload_bytes]);
            empty_frame.len()
        };

        let preamble = PreambleSequence::<M>::new(sample_rate);
        let frame_detector = FrameDetector::new(preamble, payload_capacity);

        let (sample_sender, sample_receiver) = unbounded();
        let average_power = Arc::new(Mutex::new(1.0));
        let average_power_clone = average_power.clone();

        let capture_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            let mut average_power = average_power_clone.lock().unwrap();
            ports.capture.as_slice(&ps).iter().for_each(|&sample| {
                *average_power *= 1.0 - AVG_POWER_REFRESH_FACTOR;
                *average_power += sample.powi(2) * AVG_POWER_REFRESH_FACTOR;
                sample_sender.send(sample).unwrap();
            });
        };

        audio.register(Box::new(capture_callback));
        info!("Capture demodulated data registered!");

        Self {
            modem,
            sample_receiver,
            frame_detector: Arc::new(Mutex::new(frame_detector)),
            recorded_data: Arc::new(Mutex::new(Vec::new())),
            average_power,
        }
    }

    pub fn recv(&self) -> Vec<u8> {
        loop {
            let sample = self.sample_receiver.recv().unwrap();
            self.recorded_data.lock().unwrap().push(sample);

            if let Some(frame) = self.frame_detector.lock().unwrap().update(FP::from(sample)) {
                let demodulated_frame = self.modem.demodulate(frame);
                debug!("Demodulated frame: {:?}", demodulated_frame);
                return demodulated_frame;
            }
        }
    }
}
