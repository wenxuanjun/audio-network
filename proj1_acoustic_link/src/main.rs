use std::fs::File;
use std::io::Write;
use std::time::Duration;

use proj1_acoustic_link::audio::{Audio, CreateCallback};
use proj1_acoustic_link::audio::{AudioDeactivateFlag, AudioPacket};
use proj1_acoustic_link::modem::Modem;
use proj1_acoustic_link::frame::{FrameDetector, PAYLOAD_LENGTH};

const TEST_SECONDS: usize = 3;
const TEST_SEQUENCE_LENGTH: usize = 2000;

fn main() {
    let audio = Audio::new().unwrap();

    let sample_rate = audio.sample_rate.borrow().unwrap();
    let frame_test_buffer = AudioPacket::create_buffer(sample_rate * TEST_SECONDS);

    let test_data: Vec<_> = (0..TEST_SEQUENCE_LENGTH)
        .map(|_| rand::random::<u8>() % 2)
        .collect();

    let psk = proj1_acoustic_link::modem::PSK::new(sample_rate);

    let preamble = proj1_acoustic_link::frame::PreambleSequence::new(sample_rate);

    let waiting_start_samples: Vec<_> = (0..200)
        .map(|_| rand::random::<u8>() % 2)
        .collect();

    frame_test_buffer.write_chunk(&psk.modulate(&waiting_start_samples));

    test_data.chunks(PAYLOAD_LENGTH).for_each(|frame| {
        frame_test_buffer.write_chunk(&preamble);
        frame_test_buffer.write_chunk(&psk.modulate(&frame.to_vec()));
    });

    let play_frame_callback = CreateCallback::playback(frame_test_buffer, &audio.timetick);

    let audio_input = AudioPacket::create_buffer(sample_rate * TEST_SECONDS);

    let capture_callback = CreateCallback::capture(audio_input.clone());

    println!("Beginning playback test value...");
    audio.register(play_frame_callback);
    audio.register(capture_callback);

    audio.activate();

    std::thread::sleep(Duration::from_secs(TEST_SECONDS as u64));

    println!("Stopping playback...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    let recorded_data = audio_input.read_all();
    println!("Recorded data length: {:?}", recorded_data.len());

    let correlation_test = correlate(&recorded_data, &preamble);

    write_file("correlation.py", &correlation_test);

    let mut frame_detector = FrameDetector::new(sample_rate);

    let mut payload_count = 0;

    let mut test_slice = Vec::<u8>::new();

    recorded_data.iter().enumerate().for_each(|(_, item)| {
        if let Some(frame) = frame_detector.update(*item) {
            println!("Demodulated frame: {:?}", psk.demodulate(frame));
            payload_count += 1;
            test_slice.extend(psk.demodulate(frame));
        }
    });

    println!("Payload count: {:?}", payload_count);

    println!("Test data length: {:?}, test slice length: {:?}", test_data.len(), test_slice.len());

    let mut error_count = 0;

    test_data.iter().zip(test_slice.iter()).for_each(|(a, b)| {
        if a != b {
            error_count += 1;
        }
    });

    println!("Error count: {:?}", error_count);
}

fn write_file(file_name: &str, data: &[f32]) {
    let mut file = File::create(file_name).unwrap();

    file.write_all("import numpy as np\nimport matplotlib.pyplot as plt\ny = [" .as_bytes()).unwrap();

    for item in data {
        let formatted_item = format!("{},", item);
        file.write_all(formatted_item.as_bytes()).unwrap();
    }

    file.write_all("]

x = np.arange(0, len(y), 1)

plt.plot(x, y)
plt.xlabel('Time')
plt.ylabel('Amplitude')
plt.title('Waveform')
plt.grid(True)
plt.show()".as_bytes()).unwrap();
}

fn correlate(vector: &[f32], kernel: &[f32]) -> Vec<f32> {
    use rustfft::{num_complex::Complex, FftPlanner};

    let kernel_length = kernel.len();
    let shape = vector.len() + kernel_length - 1;

    let map_zero_padding = |x: &[f32], shape: usize| {
        x.iter()
            .map(|&x| Complex::new(x, 0.0))
            .chain(std::iter::repeat(Complex::new(0.0, 0.0)))
            .take(shape)
            .collect::<Vec<_>>()
    };

    let mut vector = map_zero_padding(vector, shape);
    let mut kernel = map_zero_padding(kernel, shape);

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(shape);

    fft.process(&mut vector);
    fft.process(&mut kernel);

    let ifft = planner.plan_fft_inverse(shape);

    let mut correlation: Vec<_> = vector
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
