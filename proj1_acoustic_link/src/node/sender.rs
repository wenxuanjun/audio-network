use super::WARMUP_SEQUENCE;
use crate::audio::{Audio, AudioPacket, CreateCallback};
use crate::frame::{PreambleSequence, PAYLOAD_BYTES};
use crate::modem::Modem;

pub struct Sender;

impl Sender {
    pub fn register(audio: &'static Audio, data: &Vec<u8>) {
        let sample_rate = audio.sample_rate.borrow().unwrap();
        let sample_buffer = AudioPacket::create_buffer(0);

        let psk = crate::modem::PSK::new(sample_rate);
        let preamble = PreambleSequence::new(sample_rate);

        sample_buffer.write_chunk(&psk.modulate(&WARMUP_SEQUENCE));

        data.chunks(PAYLOAD_BYTES).for_each(|frame| {
            sample_buffer.write_chunk(&preamble);
            sample_buffer.write_chunk(&psk.modulate(&frame.to_vec()));
        });

        println!("sample_buffer length: {:?}", sample_buffer.read_all().len());

        let play_callback = CreateCallback::playback(sample_buffer, &audio.timetick);
        audio.register(play_callback);

        println!("Playback modulated data registered!");
    }
}
