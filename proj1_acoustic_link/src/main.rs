use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tunio::traits::{DriverT, InterfaceT};
use tunio::{DefaultDriver, DefaultInterface, Layer};

#[macro_use]
extern crate nolog;

const INFERFACE_NAME: &str = "anp0";

const TEST_SEND_DATA: [u8; 90] = [51, 51, 0, 0, 0, 22, 238, 19, 18, 246, 10, 152, 134, 221, 96, 0, 0, 0, 0, 36, 0, 1, 254, 128, 0, 0, 0, 0, 0, 0, 36, 14, 161, 255, 254, 136, 105, 83, 255, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 58, 0, 5, 2, 0, 0, 1, 0, 143, 0, 67, 31, 0, 0, 0, 1, 4, 0, 0, 0, 255, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 255, 0, 0, 0];

fn main() {
    let mut driver = DefaultDriver::new().unwrap();
    let if_config = DefaultDriver::if_config_builder()
        .name(INFERFACE_NAME.to_string())
        .layer(Layer::L2)
        .build()
        .unwrap();

    let interface = Arc::new(Mutex::new(DefaultInterface::new_up(&mut driver, if_config).unwrap()));

    interface.lock().unwrap().handle().add_ip("11.45.14.19/24".parse().unwrap());

    let interface_clone = interface.clone();
    std::thread::spawn(move || {
        let mut buf = vec![0u8; 4096];
        loop {
            if let Ok(n) = interface_clone.lock().unwrap().read(buf.as_mut_slice()) {
                buf.truncate(n);
                println!("{:?}", buf);
                buf.resize(4096, 0u8);
            }
        }
    });

    let interface_clone = interface.clone();
    std::thread::spawn(move || {
        loop {
            if let Ok(n) = interface_clone.lock().unwrap().write(&TEST_SEND_DATA) {
                println!("write {} bytes", n);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));

            let mut first = TEST_SEND_DATA.to_vec();
            first.resize(4096, 0u8);

            if let Ok(n) = interface_clone.lock().unwrap().write(&first) {
                println!("write {} bytes", n);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    info!("Press enter to destroy AcosticLink Network interface...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
