use audio_network::audio::Audio;
use audio_network::packet::PreambleSequence;
use audio_network::modem::BitWave;
use audio_network::node::{Receiver, Sender};
use audio_network::number::FP;

const TEST_SEQUENCE_BYTES: usize = 500;
type TargetModem = BitWave;

#[macro_use]
extern crate nolog;

#[test]
#[ignore]
fn tune_detector() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect();

    let frame_sander = Sender::<TargetModem>::new(&audio);
    let frame_receiver = Receiver::<TargetModem>::new(&audio);

    std::thread::spawn(move || {
        frame_sander.send(&test_data);
    });

    info!("Activating audio client...");
    audio.activate();

    let demodulated_data = frame_receiver.recv();
    info!("Demodulated data bytes: {:?}", demodulated_data.len());

    let sample_rate = audio.sample_rate.get().unwrap();
    let preamble = PreambleSequence::<TargetModem>::new(sample_rate);
    let correlation_test = correlate(&frame_receiver.recorded_data.lock().unwrap(), &preamble);
    plot_process_result(&correlation_test);
}

fn plot_process_result(data: &[f32]) {
    use std::{fs::File, io::Write};

    let file_name = "plot.py";
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
