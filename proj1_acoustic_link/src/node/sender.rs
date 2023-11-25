use crossbeam_channel::{unbounded, Sender as ChannelSender};
use jack::ProcessScope;

use super::WARMUP_SEQUENCE;
use crate::audio::{Audio, AudioPorts};
use crate::frame::PreambleSequence;
use crate::modem::Modem;
use crate::number::FP;

pub struct Sender<M> {
    modem: M,
    sample_sender: ChannelSender<f32>,
    preamble: Vec<FP>,
}

impl<M> Sender<M>
where
    M: Modem + Sync + Send + 'static,
{
    pub fn new(audio: &'static Audio) -> Self {
        let (sample_sender, sample_receiver) = unbounded();

        let playback_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            for sample in ports.playback.as_mut_slice(&ps) {
                *sample = sample_receiver.recv().unwrap_or(0.0)
            }
        };

        audio.register(Box::new(playback_callback));
        info!("Playback modulated data registered!");

        let sample_rate = audio.sample_rate.get().unwrap();
        let modem = M::new(sample_rate);
        let preamble = PreambleSequence::<M>::new(sample_rate);

        for &sample in modem.modulate(&WARMUP_SEQUENCE).iter() {
            sample_sender.send(FP::into(sample)).unwrap();
        }

        Self {
            modem,
            sample_sender,
            preamble,
        }
    }

    pub fn send(&self, data: &[u8]) {
        let data = {
            let mut data = data.to_vec();
            data.resize(M::PREFERED_PAYLOAD_BYTES, 0);
            data
        };

        self.preamble
            .iter()
            .chain(self.modem.modulate(&data).iter())
            .for_each(|&sample| {
                self.sample_sender.send(FP::into(sample)).unwrap();
            });
    }
}
