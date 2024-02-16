use std::path::Path;

use audio_network::audio::{Audio, AudioDeactivateFlag};
use audio_network::modem::{BitByteConverter, Ofdm, Psk, BitWave};
use audio_network::node::{ErrorCorrector, Receiver, Sender};

const TEST_SEQUENCE_BYTES: usize = 1250;

const TEST_INPUT_FILE: &str = "INPUT.txt";
const TEST_OUTPUT_FILE: &str = "OUTPUT.txt";

#[macro_use]
extern crate nolog;

#[test]
#[ignore]
fn part3_ck1_generate() {
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_INPUT_FILE);

    let test_data = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect::<Vec<u8>>();

    let test_data_bits = BitByteConverter::bytes_to_bits(&test_data)
        .iter()
        .map(|&x| x.to_string())
        .collect::<String>();

    std::fs::write(file_path.clone(), test_data_bits).unwrap();
}

#[test]
#[ignore]
fn part3_ck1_sender() {
    let audio = Audio::new().unwrap();

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_INPUT_FILE);
    info!("Reading test data from {:?}", file_path);

    let test_data_bits = std::fs::read_to_string(file_path.clone())
        .unwrap()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect::<Vec<_>>();

    let test_data = BitByteConverter::bits_to_bytes(&test_data_bits);

    let frame_sander = Sender::<Psk>::new(&audio);
    info!("Activating audio client...");
    audio.activate();

    frame_sander.send(&test_data);

    info!("Press enter to stop sending data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

#[test]
#[ignore]
fn part3_ck1_receiver() {
    let audio = Audio::new().unwrap();

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_OUTPUT_FILE);
    info!("Writing test data to {:?}", file_path);

    info!("Press enter to start receiving data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let frame_receiver = Receiver::<Psk>::new(&audio);
    info!("Activating audio client...");
    audio.activate();

    let frame_data = frame_receiver.recv();
    info!("Demodulated data length: {:?}", frame_data.len());

    let frame_data = BitByteConverter::bytes_to_bits(&frame_data)
        .iter()
        .map(|&x| x.to_string())
        .collect::<String>();

    std::fs::write(file_path.clone(), frame_data).unwrap();
}

#[test]
#[ignore]
fn part3_ck1_selfcheck() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|index| (index % 256) as u8)
        .collect();
    info!("Test data length: {:?}", test_data.len());

    let frame_sander = Sender::<Ofdm>::new(&audio);
    let frame_receiver = Receiver::<Ofdm>::new(&audio);

    info!("Activating audio client...");
    audio.activate();

    let test_data_clone = test_data.clone();
    std::thread::spawn(move || {
        frame_sander.send(&test_data_clone);
    });

    let frame_data = frame_receiver.recv();
    info!("Demodulated data length: {:?}", frame_data.len());

    info!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    info!("Demodulated data bytes: {:?}", frame_data.len());
    count_error(&test_data, &frame_data);
}

#[test]
#[ignore]
fn part4_ck1_selfcheck() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|index| (index % 256) as u8)
        .collect();

    let encoded_data = ErrorCorrector::encode(&test_data);

    let frame_sander = Sender::<Psk>::new(&audio);
    let frame_receiver = Receiver::<Psk>::new(&audio);

    info!("Activating audio client...");
    audio.activate();

    let encoded_data_clone = encoded_data.clone();
    std::thread::spawn(move || {
        frame_sander.send(&encoded_data_clone);
    });

    let frame_data = frame_receiver.recv();
    info!("Demodulated data length: {:?}", frame_data.len());

    info!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    info!("Demodulated data bytes: {:?}", frame_data.len());
    count_error(&encoded_data, &frame_data);

    let decoded_data = ErrorCorrector::decode(&frame_data);
    info!("Decoded data length: {:?}", decoded_data.len());
    count_error(&test_data, &decoded_data);
}

#[test]
#[ignore]
fn part5_ck1_selfcheck() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|index| (index % 256) as u8)
        .collect();

    let encoded_data = ErrorCorrector::encode(&test_data);

    let frame_sander = Sender::<BitWave>::new(&audio);
    let frame_receiver = Receiver::<BitWave>::new(&audio);

    info!("Activating audio client...");
    audio.activate();

    let encoded_data_clone = encoded_data.clone();
    std::thread::spawn(move || {
        frame_sander.send(&encoded_data_clone);
    });

    let frame_data = frame_receiver.recv();
    info!("Demodulated data length: {:?}", frame_data.len());

    info!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    info!("Demodulated data bytes: {:?}", frame_data.len());
    count_error(&encoded_data, &frame_data);

    let decoded_data = ErrorCorrector::decode(&frame_data);
    info!("Decoded data length: {:?}", decoded_data.len());
    count_error(&test_data, &decoded_data);
}

fn count_error(origin: &[u8], result: &[u8]) {
    let error_index: Vec<_> = BitByteConverter::bytes_to_bits(origin)
        .iter()
        .zip(BitByteConverter::bytes_to_bits(result).iter())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(index, _)| index)
        .collect();

    warn!(
        "Error bits: {:?}, Error index: {:?}",
        error_index.len(),
        error_index
    );
}
