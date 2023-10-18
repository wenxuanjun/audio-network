use std::sync::RwLock;

use jack::ProcessScope;
use super::{audio::AudioCallback, AudioPorts, AudioPacket};

pub struct CreateCallback;

impl CreateCallback {
    pub fn capture(output: AudioPacket) -> AudioCallback {
        let capture_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            output.write_chunk(ports.capture.as_slice(&ps));
        };
        Box::new(capture_callback)
    }

    pub fn playback(input: AudioPacket, timetick: &'static RwLock<u64>) -> AudioCallback {
        let playback_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            let time = *timetick.read().unwrap() as f32;
            let buffer = ports.playback.as_mut_slice(&ps);
            for (index, sample) in buffer.iter_mut().enumerate() {
                let current_sample = (index as f32 + time) as usize;
                *sample = match input.read_sample(current_sample) {
                    Some(sample) => sample,
                    None => break,
                };
            }
        };
        Box::new(playback_callback)
    }
}
