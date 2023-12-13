use proj1_acoustic_link::audio::Audio;
use proj1_acoustic_link::modem::{BitByteConverter, Modem, Ofdm};
use proj1_acoustic_link::node::{Receiver, Sender};

#[macro_use]
extern crate nolog;

const TEST_SEQUENCE_BYTES: usize = 6250;

fn main() {
    let audio = Audio::new().unwrap();

    let test_data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect();

    let frame_sander = Sender::<Ofdm>::new(&audio);
    let frame_receiver = Receiver::<Ofdm>::new(&audio);

    info!("Activating audio...");
    audio.activate();

    let test_data_clone = test_data.clone();
    std::thread::spawn(move || {
        test_data_clone
            .chunks(Ofdm::PREFERED_PAYLOAD_BYTES)
            .for_each(|chunk| {
                frame_sander.send(&chunk);
            });
    });

    let frame_count = TEST_SEQUENCE_BYTES.div_ceil(Ofdm::PREFERED_PAYLOAD_BYTES);
    let mut demodulated_data = (0..frame_count)
        .flat_map(|_| frame_receiver.recv())
        .collect::<Vec<_>>();
    demodulated_data.truncate(TEST_SEQUENCE_BYTES);

    info!("Demodulated data bytes: {:?}", demodulated_data.len());
    count_error(&test_data, &demodulated_data);
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
