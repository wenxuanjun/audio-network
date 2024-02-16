use argh::FromArgs;
use ipnet::Ipv4Net;
use std::io::{Read, Write};

use audio_network::audio::Audio;
use audio_network::modem::Ofdm;
use audio_network::node::{Receiver, Sender};

type TargetModem = Ofdm;

#[macro_use]
extern crate nolog;

const DEFAULT_INFERFACE_NAME: &str = "anp0";
const DEFAULT_IP_ADDRESS: &str = "11.45.14.19/24";

#[derive(FromArgs)]
#[argh(description = "Create an audio based network interface")]
struct Args {
    #[argh(option, short = 'i')]
    #[argh(description = "interface name of the adapter")]
    #[argh(default = "DEFAULT_INFERFACE_NAME.to_string()")]
    name: String,

    #[argh(option, short = 'a')]
    #[argh(description = "the network IP network address")]
    #[argh(default = "DEFAULT_IP_ADDRESS.to_string()")]
    address: String,
}

fn main() {
    let args: Args = argh::from_env();

    let (mut if_reader, mut if_writer) = {
        let mut if_config = tun::Configuration::default();
        let ip_network: Ipv4Net = args.address.parse().unwrap();

        if_config
            .name(args.name)
            .address(ip_network.addr())
            .netmask(ip_network.netmask())
            .layer(tun::Layer::L2)
            .up();

        let device = tun::create(&if_config).unwrap();

        device.split()
    };

    let audio = Audio::new().unwrap();

    let frame_sander = Sender::<TargetModem>::new(&audio);
    let frame_receiver = Receiver::<TargetModem>::new(&audio);

    info!("Activating audio client...");
    audio.activate();

    std::thread::spawn(move || {
        let mut buf = vec![0u8; 4096];
        loop {
            if let Ok(n) = if_reader.read(buf.as_mut_slice()) {
                buf.truncate(n);
                info!("From interface: {:?}", buf);
                frame_sander.send(&buf);
                buf.resize(4096, 0u8);
            }
        }
    });

    std::thread::spawn(move || loop {
        let frame_data = frame_receiver.recv();
        if let Ok(_) = if_writer.write(&frame_data) {
            info!("To interface: {:?}", frame_data);
        }
    });

    info!("Press enter to destroy AcosticLink Network interface...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
