use proj1_acoustic_link::audio::{Audio, AudioDeactivateFlag};
use proj1_acoustic_link::modem::{Modem, Ofdm, Psk};
use proj1_acoustic_link::node::{Receiver, Sender};

const TEST_EXTRA_WAITING: usize = 1;
const TEST_SEQUENCE_BYTES: usize = 250;

fn main() {
    let audio = Audio::new().unwrap();

    let actual_sequence_bytes = {
        let unit_payload_bytes = Ofdm::PREFERED_PAYLOAD_BYTES;
        let extra_bytes = unit_payload_bytes - TEST_SEQUENCE_BYTES % unit_payload_bytes;
        TEST_SEQUENCE_BYTES + extra_bytes
    };

    let test_data: Vec<_> = (0..actual_sequence_bytes)
        .map(|_| rand::random::<u8>())
        .collect();

    Sender::<Ofdm>::register(&audio, &test_data);
    let received_output = Receiver::<Ofdm>::register(&audio);

    println!("Activating audio...");
    audio.activate();

    let duration = ((test_data.len() * 8).div_ceil(Psk::BIT_RATE)) + TEST_EXTRA_WAITING;
    std::thread::sleep(std::time::Duration::from_secs(duration as u64));

    println!("Deactivating audio...");
    audio.deactivate(AudioDeactivateFlag::Deactivate);

    let mut received_output = received_output.lock().unwrap();

    let demodulated_data = &mut received_output.demodulated_data;
    demodulated_data.truncate(TEST_SEQUENCE_BYTES);
    println!("Demodulated data bytes: {:?}", demodulated_data.len());

    count_error(&test_data, demodulated_data);
}

fn count_error(origin: &[u8], result: &[u8]) {
    let error_index: Vec<_> = origin
        .iter()
        .zip(result.iter())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(index, _)| index)
        .collect();

    println!(
        "Error count: {:?}, Error: {:?}",
        error_index.len(),
        error_index
    );
}
