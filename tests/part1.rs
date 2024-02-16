use std::time::Duration;
use audio_network::audio::{Audio, CreateCallback};
use audio_network::audio::{AudioPacket, AudioDeactivateFlag};

const TEST_SECONDS: usize = 5;

#[test]
#[ignore]
fn part1_ck1() {
    let audio = Audio::new().unwrap();

    let sample_rate = audio.sample_rate.get().unwrap();
    let audio_input = AudioPacket::create_buffer(sample_rate * TEST_SECONDS);

    let capture_callback = CreateCallback::capture(audio_input.clone());

    audio.register(Box::new(capture_callback));

    println!("Beginning recording...");
    audio.activate();

    std::thread::sleep(Duration::from_secs(TEST_SECONDS as u64));

    println!("Restarting and cleaning up...");
    audio.deactivate(AudioDeactivateFlag::CleanRestart);

    let playback_callback = CreateCallback::playback(audio_input, &audio.timetick);

    println!("Beginning playback...");
    audio.register(Box::new(playback_callback));
    audio.activate();

    std::thread::sleep(Duration::from_secs(TEST_SECONDS as u64));

    println!("Stopping playback...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);
}

#[test]
#[ignore]
fn part1_ck2() {
    let audio = Audio::new().unwrap();

    let sample_rate = audio.sample_rate.get().unwrap();
    let audio_sample = AudioPacket::create_reader("Sample.wav");
    let audio_input = AudioPacket::create_buffer(sample_rate * TEST_SECONDS);

    let capture_callback = CreateCallback::capture(audio_input.clone());
    let playback_sample_callback = CreateCallback::playback(audio_sample, &audio.timetick);

    println!("Beginning playback...");
    audio.register(capture_callback);
    audio.register(playback_sample_callback);
    audio.activate();

    std::thread::sleep(Duration::from_secs(TEST_SECONDS as u64));

    println!("Restarting and cleaning up...");
    audio.deactivate(AudioDeactivateFlag::CleanRestart);

    let playback_buffer_callback = CreateCallback::playback(audio_input, &audio.timetick);

    println!("Beginning playback...");
    audio.register(playback_buffer_callback);
    audio.activate();

    std::thread::sleep(Duration::from_secs(TEST_SECONDS as u64));

    println!("Stopping playback...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);
}
