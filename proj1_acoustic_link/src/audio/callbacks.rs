use std::sync::RwLock;

use jack::ProcessScope;
use super::{audio::Callback, AudioPorts, AudioPacket};

pub struct AudioCallback;

impl AudioCallback {
    pub fn capture(output: AudioPacket) -> Callback {
        let capture_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
            for sample in ports.capture.as_slice(&ps).iter() {
                output.write_sample(*sample);
            }
        };

        Box::new(capture_callback)
    }

    pub fn playback(input: AudioPacket, timetick: &'static RwLock<u64>) -> Callback {
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
