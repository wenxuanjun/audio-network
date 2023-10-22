use proj1_acoustic_link::audio::{Audio, AudioDeactivateFlag};
use proj1_acoustic_link::modem::{Modem, PSK};
use proj1_acoustic_link::node::{Receiver, Sender, ErrorCorrector};

const TEST_EXTRA_WAITING: usize = 1;
const TEST_SEQUENCE_BYTES: usize = 1250;

fn main() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect();

    let encoded_data = ErrorCorrector::encode(&test_data);

    Sender::register(&audio, &encoded_data);
    let received_output = Receiver::register(&audio);

    println!("Activating audio...");
    audio.activate();

    let duration = ((encoded_data.len() * 8).div_ceil(PSK::BIT_RATE)) + TEST_EXTRA_WAITING;
    std::thread::sleep(std::time::Duration::from_secs(duration as u64));

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    let received_output = received_output.lock().unwrap();
    let demodulated_data = &received_output.demodulated_data;
    println!("Demodulated data length: {:?}", demodulated_data.len());
    count_error(&encoded_data, demodulated_data);

    let mut decoded_data = ErrorCorrector::decode(&demodulated_data);
    decoded_data.truncate(TEST_SEQUENCE_BYTES);

    println!("Decoded data length: {:?}", decoded_data.len());
    count_error(&test_data, &decoded_data);
}

fn count_error(origin: &[u8], result: &[u8]) {
    let error_index: Vec<_> = origin
        .iter()
        .zip(result.iter())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect();

    println!("Error count: {:?}, Error: {:?}", error_index.len(), error_index);
}
