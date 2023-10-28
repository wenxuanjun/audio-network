use std::marker::PhantomData;

use super::WARMUP_SEQUENCE;
use crate::audio::{Audio, AudioPacket, CreateCallback};
use crate::frame::PreambleSequence;
use crate::modem::Modem;

pub struct Sender<M> {
    modem: PhantomData<M>,
}

impl<M: Modem> Sender<M>
where
    M: Modem + Sync + Send + 'static,
{
    pub fn register(audio: &'static Audio, data: &Vec<u8>) {
        let sample_rate = audio.sample_rate.borrow().unwrap();
        let sample_buffer = AudioPacket::create_buffer(0);

        let modem = <M as Modem>::new(sample_rate);
        let payload_bytes = <M as Modem>::PREFERED_PAYLOAD_BYTES;
        let preamble = PreambleSequence::new(sample_rate);

        sample_buffer.write_chunk(&modem.modulate(&WARMUP_SEQUENCE));

        data.chunks(payload_bytes).for_each(|frame| {
            sample_buffer.write_chunk(&preamble);
            sample_buffer.write_chunk(&modem.modulate(&frame.to_vec()));
        });

        let play_callback = CreateCallback::playback(sample_buffer, &audio.timetick);
        audio.register(play_callback);

        println!("Playback modulated data registered!");
    }
}
