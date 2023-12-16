use std::sync::Arc;

use proj2_multiple_access::terminal::{Terminal, VALID_PACKET_BYTES};

#[macro_use]
extern crate nolog;

const TEST_SEQUENCE_BYTES: usize = 6250;

fn main() {
    let mac_slice1 = [0x00u8, 0x01];
    let mac_slice2 = [0x00u8, 0x02];

    let test_data = (0..TEST_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect::<Vec<_>>();

    let test_data = Arc::new(test_data);

    warn!("Test data: {:?}", test_data);

    let terminal1 = Arc::new(Terminal::new(mac_slice1));
    let terminal2 = Arc::new(Terminal::new(mac_slice2));

    terminal1.activate();
    terminal2.activate();

    let test_data_clone = test_data.clone();
    let terminal1_clone = terminal1.clone();
    let thread1 = std::thread::spawn(move || {
        test_data_clone
            .chunks(VALID_PACKET_BYTES)
            .for_each(|chunk| {
                warn!("[0] Send chunk: {:?}", chunk);
                terminal1_clone.send(&chunk, &mac_slice2);
            });
    });

    let terminal2_clone = terminal2.clone();
    let thread2 = std::thread::spawn(move || {
        let frame_count = TEST_SEQUENCE_BYTES.div_ceil(VALID_PACKET_BYTES);

        let mut result = (0..frame_count)
            .map(|_| terminal2_clone.recv())
            .collect::<Vec<_>>();

        // 按照frame.sequence顺序排序
        result.sort_by(|a, b| a.sequence.cmp(&b.sequence));

        let mut result = result
            .iter()
            .map(|x| x.payload.clone())
            .flatten()
            .collect::<Vec<_>>();

        result.truncate(TEST_SEQUENCE_BYTES);

        let diff = test_data
            .iter()
            .zip(result.iter())
            .filter(|&(x, y)| x != y)
            .count();

        warn!("[1] Result len: {}, diff: {}", result.len(), diff);
        warn!("[1] Result: {:?}", result);
    });

    thread1.join().unwrap();
    thread2.join().unwrap();
}
