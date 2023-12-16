use std::{sync::Arc, time::Duration};

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

        //let recorded = terminal1.receiver_node.recorded_data.lock().unwrap();

        //plot_process_result(&recorded);
        
        warn!("[1] Result len: {}, diff: {}", result.len(), diff);
        warn!("[1] Result: {:?}", result);
    });

    thread1.join().unwrap();
    thread2.join().unwrap();
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
