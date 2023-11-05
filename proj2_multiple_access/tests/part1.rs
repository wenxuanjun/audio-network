use std::path::Path;

use proj1_acoustic_link::audio::{Audio, AudioDeactivateFlag};
use proj1_acoustic_link::modem::Ofdm;
use proj1_acoustic_link::node::{Receiver, Sender};

const TEST_EXTRA_WAITING: usize = 1;
const TEST_SEQUENCE_BYTES: usize = 6250;

const TEST_INPUT_FILE: &str = "INPUT.bin";
const TEST_OUTPUT_FILE: &str = "OUTPUT.bin";

#[test]
fn part1_ck1_sender() {
    let audio = Audio::new().unwrap();

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_INPUT_FILE);
    println!("Reading test data from {:?}", root_dir);

    let test_data = std::fs::read(file_path.clone()).unwrap();

    let actual_sequence_bytes = Sender::<Ofdm>::register(&audio, &test_data);
    println!("Actual sequence bytes: {:?}", actual_sequence_bytes);

    println!("Press enter to start sending data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    println!("Activating audio...");
    audio.activate();

    let duration = (actual_sequence_bytes * 8).div_ceil(15000) + TEST_EXTRA_WAITING;
    std::thread::sleep(std::time::Duration::from_secs(duration as u64));

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    println!("Reading test data from {:?}", root_dir);
}

#[test]
fn part1_ck1_receiver() {
    let audio = Audio::new().unwrap();

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_OUTPUT_FILE);

    let received_output = Receiver::<Ofdm>::register(&audio);

    println!("Activating audio...");
    audio.activate();

    println!("Press enter to stop receiving data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    let mut received_output = received_output.lock().unwrap();
    let demodulated_data = &mut received_output.demodulated_data;
    println!("Demodulated data bytes: {:?}", demodulated_data.len());
    demodulated_data.truncate(TEST_SEQUENCE_BYTES);

    std::fs::write(file_path.clone(), demodulated_data).unwrap();
}
