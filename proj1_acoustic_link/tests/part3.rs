use std::path::Path;

use proj1_acoustic_link::audio::{Audio, AudioDeactivateFlag};
use proj1_acoustic_link::frame::PreambleSequence;
use proj1_acoustic_link::modem::{BitByteConverter, Modem, Ofdm, Psk};
use proj1_acoustic_link::node::{ErrorCorrector, Receiver, Sender};
use proj1_acoustic_link::number::FP;

const TEST_SEQUENCE_BYTES: usize = 1250;

const TEST_INPUT_FILE: &str = "INPUT.txt";
const TEST_OUTPUT_FILE: &str = "OUTPUT.txt";

#[macro_use]
extern crate nolog;

#[test]
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
    info!("Activating audio...");
    audio.activate();

    test_data
        .chunks(Psk::PREFERED_PAYLOAD_BYTES)
        .for_each(|chunk| {
            frame_sander.send(&chunk);
        });

    info!("Press enter to stop sending data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

#[test]
fn part3_ck1_receiver() {
    let audio = Audio::new().unwrap();

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_OUTPUT_FILE);
    info!("Writing test data to {:?}", file_path);

    info!("Press enter to start receiving data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let mut frame_receiver = Receiver::<Psk>::new(&audio);
    info!("Activating audio...");
    audio.activate();

    let frame_count = TEST_SEQUENCE_BYTES.div_ceil(Psk::PREFERED_PAYLOAD_BYTES);
    let mut demodulated_data = (0..frame_count)
        .flat_map(|_| frame_receiver.recv())
        .collect::<Vec<_>>();
    demodulated_data.truncate(TEST_SEQUENCE_BYTES);

    let demodulated_data = BitByteConverter::bytes_to_bits(&demodulated_data)
        .iter()
        .map(|&x| x.to_string())
        .collect::<String>();

    std::fs::write(file_path.clone(), demodulated_data).unwrap();
}

#[test]
fn part3_ck1_selfcheck() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect();

    let frame_sander = Sender::<Psk>::new(&audio);
    let mut frame_receiver = Receiver::<Psk>::new(&audio);

    let test_data_clone = test_data.clone();
    std::thread::spawn(move || {
        test_data_clone
            .chunks(Psk::PREFERED_PAYLOAD_BYTES)
            .for_each(|chunk| {
                frame_sander.send(&chunk);
            });
    });

    info!("Activating audio...");
    audio.activate();

    let frame_count = test_data.len().div_ceil(Psk::PREFERED_PAYLOAD_BYTES);
    let mut demodulated_data = (0..frame_count)
        .flat_map(|_| frame_receiver.recv())
        .collect::<Vec<_>>();
    demodulated_data.truncate(test_data.len());

    info!("Demodulated data bytes: {:?}", demodulated_data.len());
    count_error(&test_data, &demodulated_data);

    let sample_rate = audio.sample_rate.get().unwrap();
    let preamble = PreambleSequence::<Psk>::new(sample_rate);
    let correlation_test = correlate(&frame_receiver.recorded_data, &preamble);
    plot_process_result(&correlation_test);
}

#[test]
fn part4_ck1_selfcheck() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect();

    let encoded_data = ErrorCorrector::encode(&test_data);

    let frame_sander = Sender::<Psk>::new(&audio);
    let mut frame_receiver = Receiver::<Psk>::new(&audio);

    let encoded_data_clone = encoded_data.clone();
    std::thread::spawn(move || {
        encoded_data_clone
            .chunks(Psk::PREFERED_PAYLOAD_BYTES)
            .for_each(|chunk| {
                frame_sander.send(&chunk);
            });
    });

    println!("Activating audio...");
    audio.activate();

    let frame_count = encoded_data.len().div_ceil(Psk::PREFERED_PAYLOAD_BYTES);
    let mut demodulated_data = (0..frame_count)
        .flat_map(|_| frame_receiver.recv())
        .collect::<Vec<_>>();
    demodulated_data.truncate(encoded_data.len());

    info!("Demodulated data bytes: {:?}", demodulated_data.len());
    count_error(&encoded_data, &demodulated_data);

    let decoded_data = ErrorCorrector::decode(&demodulated_data);
    info!("Decoded data length: {:?}", decoded_data.len());
    count_error(&test_data, &decoded_data);
}

#[test]
fn part5_ck1_selfcheck() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect();

    info!("Test data length: {:?}", test_data.len());

    let encoded_data = ErrorCorrector::encode(&test_data);

    info!("Encoded data length: {:?}", encoded_data.len());

    let frame_sander = Sender::<Ofdm>::new(&audio);
    let mut frame_receiver = Receiver::<Ofdm>::new(&audio);

    println!("Activating audio...");
    audio.activate();

    let encoded_data_clone = encoded_data.clone();
    std::thread::spawn(move || {
        encoded_data_clone
            .chunks(Ofdm::PREFERED_PAYLOAD_BYTES)
            .for_each(|chunk| {
                frame_sander.send(&chunk);
            });
    });

    let frame_count = encoded_data.len().div_ceil(Ofdm::PREFERED_PAYLOAD_BYTES);
    let mut demodulated_data = (0..frame_count)
        .flat_map(|_| frame_receiver.recv())
        .collect::<Vec<_>>();
    info!("Demodulated data length: {:?}", demodulated_data.len());
    demodulated_data.truncate(encoded_data.len());

    info!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    info!("Demodulated data bytes: {:?}", demodulated_data.len());
    count_error(&encoded_data, &demodulated_data);

    let decoded_data = ErrorCorrector::decode(&demodulated_data);
    info!("Decoded data length: {:?}", decoded_data.len());
    count_error(&test_data, &decoded_data);

    let sample_rate = audio.sample_rate.get().unwrap();
    let preamble = PreambleSequence::<Ofdm>::new(sample_rate);
    let correlation_test = correlate(&frame_receiver.recorded_data, &preamble);
    plot_process_result(&correlation_test);
}

fn count_error(origin: &[u8], result: &[u8]) {
    let error_index: Vec<_> = BitByteConverter::bytes_to_bits(origin)
        .iter()
        .zip(BitByteConverter::bytes_to_bits(result).iter())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(index, _)| index)
        .collect();

    println!(
        "Error bits: {:?}, Error index: {:?}",
        error_index.len(),
        error_index
    );
}

fn plot_process_result(data: &[f32]) {
    use temp_dir::TempDir;

    let directory = TempDir::new().unwrap();
    let file_path = directory.child("plot.py");

    let header = "import numpy as np
    \nimport matplotlib.pyplot as plt
    \ny = [";

    std::fs::write(&file_path, header).unwrap();

    for item in data {
        let formatted_item = format!("{},", item);
        std::fs::write(&file_path, formatted_item.as_bytes()).unwrap();
    }

    let footer = "]
    \nx = np.arange(0, len(y), 1)
    \nplt.plot(x, y)
    \nplt.xlabel('Time')
    \nplt.ylabel('Amplitude')
    \nplt.title('Waveform')
    \nplt.grid(True)
    \nplt.show()";

    std::fs::write(&file_path, footer.as_bytes()).unwrap();

    info!("File path: {:?}", file_path);

    std::process::Command::new("python")
        .arg(file_path)
        .output()
        .expect("failed to execute process");
}

fn correlate(data: &[f32], kernel: &[FP]) -> Vec<f32> {
    use rustfft::{num_complex::Complex, FftPlanner};

    let kernel_length = kernel.len();
    let shape = data.len() + kernel_length - 1;

    let map_zero_padding = |x: &[f32], shape: usize| {
        x.iter()
            .map(|&x| Complex::new(x, 0.0))
            .chain(std::iter::repeat(Complex::new(0.0, 0.0)))
            .take(shape)
            .collect::<Vec<_>>()
    };

    let mut data = map_zero_padding(data, shape);
    let mut kernel = map_zero_padding(
        &kernel.iter().map(|&x| FP::into(x)).collect::<Vec<_>>(),
        shape,
    );

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(shape);

    fft.process(&mut data);
    fft.process(&mut kernel);

    let ifft = planner.plan_fft_inverse(shape);

    let mut correlation: Vec<_> = data
        .iter_mut()
        .zip(kernel.iter())
        .map(|(a, b)| *a * *b)
        .collect();

    ifft.process(&mut correlation);

    let correlation: Vec<_> = correlation
        .into_iter()
        .map(|x| x.re / shape as f32)
        .skip(kernel_length - 1)
        .collect();

    correlation
}
