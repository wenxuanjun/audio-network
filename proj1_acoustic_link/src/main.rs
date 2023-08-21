use std::time::Duration;
use proj1_acoustic_link::audio::{Audio, AudioCallback};
use proj1_acoustic_link::audio::{AudioPacket, AudioDeactivateFlags};

const TEST_SECONDS: usize = 10;

fn main() {
    let audio = Audio::new().unwrap();
    audio.init_client().unwrap();

    let sample_rate = audio.sample_rate.borrow().unwrap();
    let audio_sample = AudioPacket::reader("Sample.wav");
    let audio_input = AudioPacket::buffer(sample_rate * TEST_SECONDS);

    let capture_callback = AudioCallback::capture(audio_input.clone());
    let playback_sample = AudioCallback::playback(audio_sample, &audio.timetick);

    println!("Beginning playback...");
    audio.register(capture_callback);
    audio.register(playback_sample);
    audio.activate();

    std::thread::sleep(Duration::from_secs(TEST_SECONDS as u64));

    println!("Restarting and cleaning up...");
    audio.deactivate(AudioDeactivateFlags::CleanRestart);

    let playback_buffer = AudioCallback::playback(audio_input, &audio.timetick);

    println!("Beginning playback...");
    audio.register(playback_buffer);
    audio.activate();

    std::thread::sleep(Duration::from_secs(TEST_SECONDS as u64));

    println!("Stopping playback...");
    audio.deactivate(AudioDeactivateFlags::Deactivate);
}
