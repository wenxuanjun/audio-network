use std::path::Path;

use proj1_acoustic_link::audio::{Audio, AudioDeactivateFlag};
use proj1_acoustic_link::frame::PreambleSequence;
use proj1_acoustic_link::modem::{BitByteConverter, Ofdm, Psk};
use proj1_acoustic_link::node::{ErrorCorrector, Receiver, Sender};
use proj1_acoustic_link::number::FP;

const TEST_EXTRA_WAITING: usize = 1;
const TEST_SEQUENCE_BYTES: usize = 1250;

const TEST_INPUT_FILE: &str = "INPUT.txt";
const TEST_OUTPUT_FILE: &str = "OUTPUT.txt";

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
    println!("Reading test data from {:?}", root_dir);

    let test_data_bits = std::fs::read_to_string(file_path.clone())
        .unwrap()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect::<Vec<_>>();

    let test_data = BitByteConverter::bits_to_bytes(&test_data_bits);

    let actual_sequence_bytes = Sender::<Psk>::register(&audio, &test_data);
    println!("Actual sequence bytes: {:?}", actual_sequence_bytes);

    println!("Press enter to start sending data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    println!("Activating audio...");
    audio.activate();

    let duration = ((actual_sequence_bytes * 8).div_ceil(Psk::BIT_RATE)) + TEST_EXTRA_WAITING;
    std::thread::sleep(std::time::Duration::from_secs(duration as u64));

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    println!("Reading test data from {:?}", root_dir);
}

#[test]
fn part3_ck1_receiver() {
    let audio = Audio::new().unwrap();

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_OUTPUT_FILE);

    let received_output = Receiver::<Psk>::register(&audio);

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

    let demodulated_data = BitByteConverter::bytes_to_bits(demodulated_data)
        .iter()
        .map(|&x| x.to_string())
        .collect::<String>();

    std::fs::write(file_path.clone(), demodulated_data).unwrap();
}

#[test]
fn part3_ck1_selfcheck() {
    let audio = Audio::new().unwrap();
    let sample_rate = audio.sample_rate.borrow().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect();

    let actual_sequence_bytes = Sender::<Psk>::register(&audio, &test_data);
    let received_output = Receiver::<Psk>::register(&audio);
    println!("Actual sequence bytes: {:?}", actual_sequence_bytes);

    println!("Activating audio...");
    audio.activate();

    let duration = ((actual_sequence_bytes * 8).div_ceil(Psk::BIT_RATE)) + TEST_EXTRA_WAITING;
    std::thread::sleep(std::time::Duration::from_secs(duration as u64));

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    let mut received_output = received_output.lock().unwrap();

    let demodulated_data = &mut received_output.demodulated_data;
    println!("Demodulated data bytes: {:?}", demodulated_data.len());
    demodulated_data.truncate(TEST_SEQUENCE_BYTES);

    count_error(&test_data, demodulated_data);

    let preamble = PreambleSequence::new(sample_rate);
    let correlation_test = correlate(&received_output.recorded_data, &preamble);
    plot_process_result("correlation.py", &correlation_test);
}

#[test]
fn part4_ck1_selfcheck() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect();

    let encoded_data = ErrorCorrector::encode(&test_data);
    let origin_encoded_length = encoded_data.len();

    let actual_sequence_bytes = Sender::<Psk>::register(&audio, &encoded_data);
    let received_output = Receiver::<Psk>::register(&audio);
    println!("Actual sequence bytes: {:?}", actual_sequence_bytes);

    println!("Activating audio...");
    audio.activate();

    let duration = ((actual_sequence_bytes * 8).div_ceil(Psk::BIT_RATE)) + TEST_EXTRA_WAITING;
    std::thread::sleep(std::time::Duration::from_secs(duration as u64));

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    let mut received_output = received_output.lock().unwrap();
    let demodulated_data = &mut received_output.demodulated_data;
    println!("Demodulated data bytes: {:?}", demodulated_data.len());
    count_error(&encoded_data, demodulated_data);

    demodulated_data.truncate(origin_encoded_length);
    let mut decoded_data = ErrorCorrector::decode(&demodulated_data);
    println!("Decoded data length: {:?}", decoded_data.len());

    decoded_data.truncate(TEST_SEQUENCE_BYTES);
    count_error(&test_data, &decoded_data);
}

#[test]
fn part5_ck1_selfcheck() {
    let audio = Audio::new().unwrap();
    let sample_rate = audio.sample_rate.borrow().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect();

    let encoded_data = ErrorCorrector::encode(&test_data);
    let origin_encoded_length = encoded_data.len();

    let actual_sequence_bytes = Sender::<Ofdm>::register(&audio, &encoded_data);
    let received_output = Receiver::<Ofdm>::register(&audio);
    println!("Actual sequence bytes: {:?}", actual_sequence_bytes);

    println!("Activating audio...");
    audio.activate();

    let duration = ((encoded_data.len() * 8).div_ceil(1000)) + TEST_EXTRA_WAITING;
    std::thread::sleep(std::time::Duration::from_secs(duration as u64));

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    let mut received_output = received_output.lock().unwrap();
    let demodulated_data = &mut received_output.demodulated_data;
    println!("Demodulated data bytes: {:?}", demodulated_data.len());

    count_error(&encoded_data, demodulated_data);

    demodulated_data.truncate(origin_encoded_length);
    let mut decoded_data = ErrorCorrector::decode(&demodulated_data);
    println!("Decoded data length: {:?}", decoded_data.len());

    decoded_data.truncate(TEST_SEQUENCE_BYTES);
    count_error(&test_data, &decoded_data);

    let preamble = PreambleSequence::new(sample_rate);
    let correlation_test = correlate(&received_output.recorded_data, &preamble);
    plot_process_result("correlation.py", &correlation_test);
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

fn plot_process_result(file_name: &str, data: &[f32]) {
    use std::{fs::File, io::Write};

    let mut file = File::create(file_name).unwrap();

    let header = "import numpy as np
    \nimport matplotlib.pyplot as plt
    \ny = [";

    file.write_all(header.as_bytes()).unwrap();

    for item in data {
        let formatted_item = format!("{},", item);
        file.write_all(formatted_item.as_bytes()).unwrap();
    }

    let footer = "]
    \nx = np.arange(0, len(y), 1)
    \nplt.plot(x, y)
    \nplt.xlabel('Time')
    \nplt.ylabel('Amplitude')
    \nplt.title('Waveform')
    \nplt.grid(True)
    \nplt.show()";

    file.write_all(footer.as_bytes()).unwrap();

    std::process::Command::new("python")
        .arg(file_name)
        .output()
        .expect("failed to execute process");

    std::fs::remove_file(file_name).unwrap();
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
