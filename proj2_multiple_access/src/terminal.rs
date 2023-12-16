use crossbeam_channel::{after, select, tick, bounded};
use crossbeam_channel::{Receiver as ChannelReceiver, Sender as ChannelSender};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::corrupted::{CrcWrapper, CRC_BYTES};
use proj1_acoustic_link::audio::Audio;
use proj1_acoustic_link::modem::{Modem, Ofdm};
use proj1_acoustic_link::node::{Receiver, Sender};

const ACK_MAGIC_NUMBER: [u8; 6] = [0x11, 0x45, 0x14, 0x19, 0x19, 0x81];
const ACK_PAYLOAD_BYTES: usize = ACK_MAGIC_NUMBER.len();
const MAC_ADDRESS_BYTES: usize = 2;
const SEQUENCE_BYTES: usize = std::mem::size_of::<u32>();
const DATA_FRAME_BYTES: usize = Ofdm::PREFERED_PAYLOAD_BYTES - CRC_BYTES;
pub const VALID_PACKET_BYTES: usize = DATA_FRAME_BYTES - MAC_ADDRESS_BYTES * 2 - SEQUENCE_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress([u8; MAC_ADDRESS_BYTES]);

impl MacAddress {
    pub fn new(mac_address: [u8; MAC_ADDRESS_BYTES]) -> Self {
        Self(mac_address)
    }
}

struct TerminalChannelPair<T> {
    sender: ChannelSender<T>,
    receiver: ChannelReceiver<T>,
}

impl<T> TerminalChannelPair<T> {
    fn new() -> Self {
        let (sender, receiver) = bounded(0);
        Self { sender, receiver }
    }
}

#[derive(Debug, Clone)]
pub struct TerminalDataFrame {
    pub sequence: u32,
    pub payload: Vec<u8>,
    source: MacAddress,
    destination: MacAddress,
}

impl TerminalDataFrame {
    pub fn new(source: MacAddress, destination: MacAddress, sequence: u32, payload: &[u8]) -> Self {
        let payload = payload.to_vec();
        assert_eq!(payload.len(), VALID_PACKET_BYTES);

        Self {
            source,
            destination,
            sequence,
            payload,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() != DATA_FRAME_BYTES {
            return None;
        }

        let (source, data) = data.split_at(MAC_ADDRESS_BYTES);
        let (destination, data) = data.split_at(MAC_ADDRESS_BYTES);
        let (sequence, payload) = data.split_at(SEQUENCE_BYTES);

        let data_frame = Self {
            source: MacAddress::new([source[0], source[1]]),
            destination: MacAddress::new([destination[0], destination[1]]),
            sequence: u32::from_be_bytes(sequence.try_into().unwrap()),
            payload: payload.to_vec(),
        };

        Some(data_frame)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(DATA_FRAME_BYTES);
        result.extend_from_slice(&self.source.0);
        result.extend_from_slice(&self.destination.0);
        result.extend_from_slice(&self.sequence.to_be_bytes());
        result.extend_from_slice(&self.payload);
        result
    }
}

struct AckPayload;

impl AckPayload {
    pub fn create() -> Vec<u8> {
        let random_numbers: Vec<u8> = (0..VALID_PACKET_BYTES - ACK_MAGIC_NUMBER.len())
            .map(|_| rand::random::<u8>())
            .collect();

        [&ACK_MAGIC_NUMBER[..], &random_numbers[..]].concat()
    }

    pub fn validate(data: &TerminalDataFrame) -> bool {
        assert_eq!(data.payload.len(), VALID_PACKET_BYTES);

        let (magic_number, _) = data.payload.split_at(ACK_PAYLOAD_BYTES);
        return magic_number == ACK_MAGIC_NUMBER;
    }
}

enum SenderChannelData {
    Data(TerminalDataFrame),
    Ack(TerminalDataFrame),
}

pub struct Terminal {
    mac_address: MacAddress,
    running_state: Arc<AtomicBool>,
    sender_channel: TerminalChannelPair<SenderChannelData>,
    receiver_channel: TerminalChannelPair<TerminalDataFrame>,
    sender_node: Arc<Sender<Ofdm>>,
    receiver_node: Arc<Receiver<Ofdm>>,
    current_sequence: AtomicUsize,
    received_acks: Arc<Mutex<Vec<u32>>>,
    received_sequences: Arc<Mutex<Vec<u32>>>,
}

impl Terminal {
    pub fn new(mac_address: [u8; MAC_ADDRESS_BYTES]) -> Self {
        let audio = Audio::new().unwrap();
        audio.activate();

        Self {
            mac_address: MacAddress::new(mac_address),
            running_state: Arc::new(AtomicBool::new(true)),
            sender_channel: TerminalChannelPair::new(),
            receiver_channel: TerminalChannelPair::new(),
            sender_node: Arc::new(Sender::<Ofdm>::new(&audio)),
            receiver_node: Arc::new(Receiver::<Ofdm>::new(&audio)),
            current_sequence: AtomicUsize::new(0),
            received_acks: Arc::new(Mutex::new(Vec::new())),
            received_sequences: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn send(&self, data: &[u8], destination: &[u8; MAC_ADDRESS_BYTES]) {
        let data = {
            let mut data = data.to_vec();
            data.resize(VALID_PACKET_BYTES, 0);
            data
        };

        let sequence = self.current_sequence.fetch_add(1, Ordering::Relaxed) as u32;

        let data_frame = TerminalDataFrame::new(
            self.mac_address,
            MacAddress::new(*destination),
            sequence,
            &data,
        );

        std::thread::sleep(Duration::from_millis(300));

        self.sender_channel
            .sender
            .send(SenderChannelData::Data(data_frame))
            .unwrap();
    }

    pub fn recv(&self) -> TerminalDataFrame {
        self.receiver_channel.receiver.recv().unwrap()
    }

    pub fn activate(&self) {
        self.running_state.store(true, Ordering::Relaxed);
        self.active_sender();
        self.active_receiver();
    }

    fn active_sender(&self) {
        let sender_node = self.sender_node.clone();
        let running_state = self.running_state.clone();
        let sender_channel_receiver = self.sender_channel.receiver.clone();
        let received_acks = self.received_acks.clone();

        //let average_power = self.receiver_node.average_power.clone();

        std::thread::spawn(move || loop {
            if !running_state.load(Ordering::Relaxed) {
                break;
            }

            info!("Looping to send data...");

            let channel_data = sender_channel_receiver.recv().unwrap();

            match channel_data {
                SenderChannelData::Data(data_frame) => {
                    let encoded_frame = CrcWrapper::encode(&data_frame.to_bytes());

                    /*loop {
                        if average_power.colliding() {
                            std::thread::sleep(Duration::from_millis(rand::random::<u64>() % 20));
                        } else {
                            break;
                        }
                    }*/

                    warn!("Sending data frame: {:?}", data_frame.sequence);
                    sender_node.send(&encoded_frame);

                    let sender_node = sender_node.clone();
                    let running_state = running_state.clone();
                    let received_acks = received_acks.clone();
                    //let average_power = average_power.clone();

                    let ticker = tick(Duration::from_millis(600));
                    let timeout = after(Duration::from_millis(2000));

                    loop {
                        select! {
                            recv(ticker) -> _ => {
                                if received_acks
                                    .lock()
                                    .unwrap()
                                    .iter()
                                    .position(|&x| x == data_frame.sequence)
                                    .is_some()
                                {
                                    break;
                                }
                            
                                /*loop {
                                    if average_power.colliding() {
                                        std::thread::sleep(Duration::from_millis(
                                            rand::random::<u64>() % 20,
                                        ));
                                    } else {
                                        break;
                                    }
                                }*/

                                warn!("Resending data frame: {:?}", data_frame.sequence);
                                sender_node.send(&encoded_frame);
                            },
                            recv(timeout) -> _ => {
                                error!("Maximum retries for {:?} reached!", data_frame.sequence);
                                running_state.store(false, Ordering::Relaxed);
                                break;
                            },
                        }
                    }
                }
                SenderChannelData::Ack(data_frame) => {
                    let encoded_frame = CrcWrapper::encode(&data_frame.to_bytes());
                    sender_node.send(&encoded_frame);
                }
            };
        });
    }

    fn active_receiver(&self) {
        let mac_address = self.mac_address.clone();
        let receiver_node = self.receiver_node.clone();
        let running_state = self.running_state.clone();
        let sender_channel_sender = self.sender_channel.sender.clone();
        let receiver_channel_sender = self.receiver_channel.sender.clone();
        let received_acks = self.received_acks.clone();
        let received_sequences = self.received_sequences.clone();

        std::thread::spawn(move || loop {
            if !running_state.load(Ordering::Relaxed) {
                break;
            }

            let received = receiver_node.recv();

            if let Some(received) = CrcWrapper::decode(&received) {
                let data_frame = TerminalDataFrame::from_bytes(&received).unwrap();

                if data_frame.destination == mac_address {
                    if AckPayload::validate(&data_frame) {
                        received_acks.lock().unwrap().push(data_frame.sequence);
                    } else {
                        let ack_payload = AckPayload::create();

                        let ack_data_frame = TerminalDataFrame::new(
                            mac_address,
                            data_frame.source,
                            data_frame.sequence,
                            &ack_payload,
                        );

                        if !received_sequences
                            .lock()
                            .unwrap()
                            .contains(&data_frame.sequence)
                        {
                            receiver_channel_sender.send(data_frame.clone()).unwrap();
                            received_sequences.lock().unwrap().push(data_frame.sequence);
                        }

                        warn!("Want to send ack of: {:?}", ack_data_frame.sequence);

                        sender_channel_sender
                            .send(SenderChannelData::Ack(ack_data_frame))
                            .unwrap();
                    }
                }
            }
        });
    }

    pub fn deactivate(&self) {
        self.running_state.store(false, Ordering::Relaxed);
    }
}
