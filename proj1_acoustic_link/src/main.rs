use std::time::Duration;
use jack::ProcessScope;
use proj1_acoustic_link::audio::{Audio, AudioPorts};
use proj1_acoustic_link::audio::{AudioPacket, AudioDeactivateFlags};

const TEST_SECONDS: usize = 10;

fn main() {
    let audio = Audio::new().unwrap();
    audio.init_client().unwrap();

    let sample_rate = audio.sample_rate.borrow().unwrap();
    let audio_input = AudioPacket::buffer(sample_rate * TEST_SECONDS);

    let audio_input_clone = audio_input.clone();
    let capture_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
        for sample in ports.capture.as_slice(&ps).iter() {
            audio_input_clone.write_sample(*sample);
        }
    };

    audio.register(Box::new(capture_callback));

    println!("Beginning recording...");
    audio.activate();

    std::thread::sleep(Duration::from_secs(TEST_SECONDS as u64));

    println!("Restarting and cleaning up...");
    audio.deactivate(AudioDeactivateFlags::CleanRestart);

    let timetick = &audio.timetick;
    let playback_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
        let time = *timetick.read().unwrap() as f32;
        let buffer = ports.playback.as_mut_slice(&ps);
        for (index, sample) in buffer.iter_mut().enumerate() {
            let current_sample = (index as f32 + time) as usize;
            *sample = match audio_input.read_sample(current_sample) {
                Some(sample) => sample,
                None => break,
            };
        }
    };

    println!("Beginning playback...");
    audio.register(Box::new(playback_callback));
    audio.activate();

    std::thread::sleep(Duration::from_secs(TEST_SECONDS as u64));

    println!("Stopping playback...");
    audio.deactivate(AudioDeactivateFlags::Deactivate);
}
