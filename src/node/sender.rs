use crossbeam_channel::{unbounded, Sender as ChannelSender};
use jack::ProcessScope;

use super::{FrameManager, WARMUP_SEQUENCE};
use crate::audio::{Audio, AudioPorts};
use crate::modem::Modem;
use crate::number::FP;
use crate::packet::PreambleSequence;

pub struct Sender<M> {
    modem: M,
    preamble: Vec<FP>,
    sample_sender: ChannelSender<f32>,
}

impl<M> Sender<M>
where
    M: Modem + Sync + Send + 'static,
{
    pub fn new(audio: &'static Audio) -> Self {
        let (sample_sender, sample_receiver) = unbounded();

        let sample_rate = audio.sample_rate.get().unwrap();
        let modem = <M as Modem>::new(sample_rate);
        let preamble = PreambleSequence::<M>::new(sample_rate);

        let playback_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            for sample in ports.playback.as_mut_slice(&ps) {
                *sample = sample_receiver.try_recv().unwrap_or(0.0)
            }
        };

        audio.register(Box::new(playback_callback));
        info!("Playback modulated data registered!");

        modem.modulate(&WARMUP_SEQUENCE).iter().for_each(|&sample| {
            sample_sender.send(FP::into(sample)).unwrap();
        });

        Self {
            modem,
            preamble,
            sample_sender,
        }
    }

    pub fn send(&self, frame: &[u8]) {
        let packets = FrameManager::<M>::construct(&frame);

        packets.iter().for_each(|packet| {
            self.preamble
                .iter()
                .chain(self.modem.modulate(&packet).iter())
                .for_each(|&sample| {
                    self.sample_sender.send(FP::into(sample)).unwrap();
                });
        });
    }
}
