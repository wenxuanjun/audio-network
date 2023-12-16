use argh::FromArgs;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tunio::traits::{DriverT, InterfaceT};
use tunio::{DefaultDriver, DefaultInterface, Layer};

use proj1_acoustic_link::audio::Audio;
use proj1_acoustic_link::modem::BitWave;
use proj1_acoustic_link::node::{Receiver, Sender};

type TargetModem = BitWave;

#[macro_use]
extern crate nolog;

const DEFAULT_INFERFACE_NAME: &str = "anp0";
const DEFAULT_IP_ADDRESS: &str = "11.45.14.19/24";

#[derive(FromArgs)]
#[argh(description = "AcousticLink Network adapter")]
struct Args {
    #[argh(option, short = 'i')]
    #[argh(description = "interface name of the adapter")]
    #[argh(default = "DEFAULT_INFERFACE_NAME.to_string()")]
    name: String,

    #[argh(option, short = 'a')]
    #[argh(description = "the network IP address")]
    #[argh(default = "DEFAULT_IP_ADDRESS.to_string()")]
    address: String,
}

fn main() {
    let args: Args = argh::from_env();

    let interface = {
        let if_config = DefaultDriver::if_config_builder()
            .name(args.name)
            .layer(Layer::L2)
            .build()
            .unwrap();
        let mut driver = DefaultDriver::new().unwrap();
        let interface = DefaultInterface::new_up(&mut driver, if_config).unwrap();
        Arc::new(Mutex::new(interface))
    };

    interface
        .lock()
        .unwrap()
        .handle()
        .add_ip(args.address.parse().unwrap());

    let audio = Audio::new().unwrap();

    let frame_sander = Sender::<TargetModem>::new(&audio);
    let frame_receiver = Receiver::<TargetModem>::new(&audio);

    info!("Activating audio client...");
    audio.activate();

    let interface_clone = interface.clone();
    std::thread::spawn(move || {
        let mut buf = vec![0u8; 4096];
        loop {
            if let Ok(n) = interface_clone.lock().unwrap().read(buf.as_mut_slice()) {
                buf.truncate(n);
                info!("From interface: {:?}", buf);
                frame_sander.send(&buf);
                buf.resize(4096, 0u8);
            }
        }
    });

    std::thread::spawn(move || loop {
        let frame_data = frame_receiver.recv();
        if let Ok(_) = interface.lock().unwrap().write(&frame_data) {
            info!("To interface: {:?}", frame_data);
        }
    });

    info!("Press enter to destroy AcosticLink Network interface...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
