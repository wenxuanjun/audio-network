use std::path::Path;

use proj1_acoustic_link::audio::{Audio, AudioDeactivateFlag};
use proj1_acoustic_link::frame::PreambleSequence;
use proj1_acoustic_link::modem::{Modem, PSK, BitByteConverter};
use proj1_acoustic_link::node::{Receiver, Sender};

const TEST_EXTRA_WAITING: usize = 1;
const TEST_SEQUENCE_BYTES: usize = 1250;

const TEST_INPUT_FILE: &str = "INPUT.txt";
const TEST_OUTPUT_FILE: &str = "OUTPUT.txt";

#[test]
fn part3_ck1_generate() {
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_INPUT_FILE);

    let test_data = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>() % 2)
        .collect::<Vec<u8>>()
        .iter()
        .map(|&x| x.to_string())
        .collect::<String>();

    std::fs::write(file_path.clone(), test_data).unwrap();
}

#[test]
fn part3_ck1_sender() {
    let audio = Audio::new().unwrap();

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file_path = root_dir.join(TEST_INPUT_FILE);
    println!("Reading test data from {:?}", root_dir);

    let test_data = std::fs::read_to_string(file_path.clone())
        .unwrap()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect::<Vec<_>>();

    Sender::register(&audio, &test_data);

    println!("Press enter to start sending data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    println!("Activating audio...");
    audio.activate();

    let duration = ((test_data.len() * 8).div_ceil(PSK::BIT_RATE)) + TEST_EXTRA_WAITING;
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

    let received_output = Receiver::register(&audio);

    println!("Activating audio...");
    audio.activate();

    println!("Press enter to stop receiving data...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    let received_output = received_output.lock().unwrap();
    let demodulated_data = &received_output.demodulated_data;
    println!("Demodulated data length: {:?}", demodulated_data.len());

    let demodulated_data = demodulated_data
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

    Sender::register(&audio, &test_data);
    let received_output = Receiver::register(&audio);

    println!("Activating audio...");
    audio.activate();

    let duration = ((test_data.len() * 8).div_ceil(PSK::BIT_RATE)) + TEST_EXTRA_WAITING;
    std::thread::sleep(std::time::Duration::from_secs(duration as u64));

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    let mut received_output = received_output.lock().unwrap();

    let demodulated_data = &mut received_output.demodulated_data;
    demodulated_data.truncate(TEST_SEQUENCE_BYTES);
    println!("Demodulated data length: {:?}", demodulated_data.len());

    count_error(&test_data, demodulated_data);

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
        .map(|(i, _)| i)
        .collect();

    println!("Error count: {:?}, Error: {:?}", error_index.len(), error_index);
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

fn correlate(data: &[f32], kernel: &[f32]) -> Vec<f32> {
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
    let mut kernel = map_zero_padding(kernel, shape);

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
