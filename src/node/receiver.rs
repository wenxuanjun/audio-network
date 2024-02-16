use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Receiver as ChannelReceiver};
use jack::ProcessScope;

use crate::audio::{Audio, AudioPorts};
use crate::modem::Modem;
use crate::number::FP;
use crate::packet::{PacketDetector, PreambleSequence};

use super::FrameManager;

#[derive(Clone)]
pub struct AveragePower(Arc<Mutex<f32>>);

impl AveragePower {
    const REFRESH_FACTOR: f32 = 0.85;
    const COLLISION_THRESHOLD: f32 = 2.5e-4;

    fn new() -> Self {
        Self(Arc::new(Mutex::new(1.0)))
    }

    fn update(&self, sample: f32) {
        let mut average_power = self.0.lock().unwrap();
        *average_power *= 1.0 - Self::REFRESH_FACTOR;
        *average_power += sample.powi(2) * Self::REFRESH_FACTOR;
    }

    pub fn colliding(&self) -> bool {
        let average_power = self.0.lock().unwrap();
        *average_power > Self::COLLISION_THRESHOLD
    }
}

pub struct Receiver<M> {
    pub average_power: AveragePower,
    pub recorded_data: Arc<Mutex<Vec<f32>>>,
    modem: M,
    sample_receiver: ChannelReceiver<f32>,
    packet_detector: Arc<Mutex<PacketDetector>>,
    frame_manager: Arc<Mutex<FrameManager<M>>>,
}

impl<M> Receiver<M>
where
    M: Modem + Sync + Send + 'static,
{
    pub fn new(audio: &'static Audio) -> Self {
        let average_power = AveragePower::new();
        let (sample_sender, sample_receiver) = unbounded();

        let average_power_clone = average_power.clone();
        let capture_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            ports.capture.as_slice(&ps).iter().for_each(|&sample| {
                average_power_clone.update(sample);
                sample_sender.send(sample).unwrap();
            });
        };

        audio.register(Box::new(capture_callback));
        info!("Capture demodulated data registered!");

        let recorded_data = Arc::new(Mutex::new(Vec::new()));
        let frame_manager = Arc::new(Mutex::new(FrameManager::<M>::new()));
        let (modem, packet_detector) = Self::create_packet_detector(audio);

        Self {
            modem,
            sample_receiver,
            packet_detector,
            recorded_data,
            average_power,
            frame_manager,
        }
    }

    pub fn recv(&self) -> Vec<u8> {
        loop {
            let sample = self.sample_receiver.recv().unwrap();
            self.recorded_data.lock().unwrap().push(sample);

            if let Some(packet) = self
                .packet_detector
                .lock()
                .unwrap()
                .update(FP::from(sample))
            {
                if let Some(frame) = self
                    .frame_manager
                    .lock()
                    .unwrap()
                    .update(&self.modem.demodulate(&packet))
                {
                    debug!("Frame received: {:?}", frame);
                    return frame;
                }
            }
        }
    }

    fn create_packet_detector(audio: &'static Audio) -> (M, Arc<Mutex<PacketDetector>>) {
        let sample_rate = audio.sample_rate.get().unwrap();
        let modem = <M as Modem>::new(sample_rate);

        let packet_detector = {
            let payload_capacity = {
                let payload_bytes = M::MIN_MODULATE_BYTES;
                let empty_packet = modem.modulate(&vec![0; payload_bytes]);
                empty_packet.len()
            };
            let preamble = PreambleSequence::<M>::new(sample_rate);
            let packet_detector = PacketDetector::new(preamble, payload_capacity);
            Arc::new(Mutex::new(packet_detector))
        };

        (modem, packet_detector)
    }
}
