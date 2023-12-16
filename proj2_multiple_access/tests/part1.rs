use std::path::Path;

use proj1_acoustic_link::audio::Audio;
use proj1_acoustic_link::modem::{Ofdm, Modem};
use proj1_acoustic_link::node::{Receiver, Sender};

const TEST_SEQUENCE_BYTES: usize = 6250;

const TEST_INPUT_FILE: &str = "INPUT.bin";
const TEST_OUTPUT_FILE: &str = "OUTPUT.bin";

#[macro_use]
extern crate nolog;

#[test]
fn part1_ck1_sender() {
    let audio = Audio::new().unwrap();

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_INPUT_FILE);
    info!("Reading test data from {:?}", file_path);

    let test_data = std::fs::read(file_path.clone()).unwrap();

    let frame_sander = Sender::<Ofdm>::new(&audio);
    info!("Activating audio client...");
    audio.activate();

    test_data
        .chunks(Ofdm::PREFERED_PAYLOAD_BYTES)
        .for_each(|chunk| {
            frame_sander.send(&chunk);
        });

    info!("Press enter to stop sending data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

#[test]
fn part1_ck1_receiver() {
    let audio = Audio::new().unwrap();

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_OUTPUT_FILE);
    info!("Writing test data to {:?}", file_path);

    info!("Press enter to start receiving data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let mut frame_receiver = Receiver::<Ofdm>::new(&audio);
    info!("Activating audio client...");
    audio.activate();

    let frame_count = TEST_SEQUENCE_BYTES.div_ceil(Ofdm::PREFERED_PAYLOAD_BYTES);
    let mut demodulated_data = (0..frame_count)
        .flat_map(|_| frame_receiver.recv())
        .collect::<Vec<_>>();
    demodulated_data.truncate(TEST_SEQUENCE_BYTES);

    std::fs::write(file_path.clone(), demodulated_data).unwrap();
}
