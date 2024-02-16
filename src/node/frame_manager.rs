use crate::modem::Modem;
use std::marker::PhantomData;

const MIN_VALID_FRAME_LENGTH: usize = 1;
const FRAME_PREAMBLE: [u8; 4] = [0b10101010, 0b10101010, 0b10101010, 0b10101011];

enum FrameManagerState {
    Waiting,
    Frame,
}

pub struct FrameManager<M> {
    phantom: PhantomData<M>,
    buffer: Vec<u8>,
    frame_length: u16,
    current_state: FrameManagerState,
}

impl<M> FrameManager<M>
where
    M: Modem + Sync + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            buffer: Vec::new(),
            frame_length: 0,
            current_state: FrameManagerState::Waiting,
        }
    }

    pub fn construct(frame: &[u8]) -> Vec<Vec<u8>> {
        let mut result = [
            &FRAME_PREAMBLE[..],
            &u16::to_ne_bytes(frame.len() as u16),
            frame,
        ]
        .concat();

        let packet_length = <M as Modem>::MIN_MODULATE_BYTES;
        let packet_num = result.len().div_ceil(packet_length);
        result.resize(packet_num * packet_length, 0);

        result.chunks(packet_length).map(Vec::from).collect()
    }

    pub fn update(&mut self, packet: &[u8]) -> Option<Vec<u8>> {
        assert!(
            packet.len()
                >= FRAME_PREAMBLE.len()
                    + std::mem::size_of_val(&self.frame_length)
                    + MIN_VALID_FRAME_LENGTH,
        );

        match self.current_state {
            FrameManagerState::Waiting => {
                let (preamble, packet) = packet.split_at(FRAME_PREAMBLE.len());

                if preamble != FRAME_PREAMBLE {
                    return None;
                }

                let (frame_length, packet) =
                    packet.split_at(std::mem::size_of_val(&self.frame_length));

                self.buffer.extend(packet);
                self.frame_length = u16::from_ne_bytes(frame_length.try_into().unwrap());

                if (self.frame_length as usize + FRAME_PREAMBLE.len())
                    <= <M as Modem>::MIN_MODULATE_BYTES
                {
                    let result = self.buffer[..self.frame_length as usize].to_vec();
                    self.buffer.clear();

                    return Some(result);
                }

                self.current_state = FrameManagerState::Frame;
                None
            }
            FrameManagerState::Frame => {
                self.buffer.extend(packet);

                if self.buffer.len() < self.frame_length as usize {
                    return None;
                }

                self.current_state = FrameManagerState::Waiting;

                let result = self.buffer[..self.frame_length as usize].to_vec();
                self.buffer.clear();
                Some(result)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::modem::{BitWave, Ofdm};

    const TEST_EXTEA_BYTES: usize = 37;
    const TEST_REDUCE_BYTES: usize = 17;

    #[test]
    fn test_frame_manager_inexact() {
        let packet_length = <BitWave as Modem>::MIN_MODULATE_BYTES;
        let mut frame_manager = FrameManager::<BitWave>::new();

        let origin = (0..packet_length * 20 + TEST_EXTEA_BYTES)
            .map(|index| index as u8)
            .collect::<Vec<_>>();
        info!("Origin length: {:?}", origin.len());

        let packets = FrameManager::<BitWave>::construct(&origin);

        for packet in packets {
            if let Some(frame) = frame_manager.update(&packet) {
                info!("Frame length: {:?}", frame.len());
                assert_eq!(origin, frame);
                return;
            }
        }

        unreachable!("Must be able to get frame response!");
    }

    #[test]
    fn test_frame_manager_small_size() {
        let packet_length = <Ofdm as Modem>::MIN_MODULATE_BYTES;
        let mut frame_manager = FrameManager::<Ofdm>::new();

        let origin = (0..packet_length - TEST_REDUCE_BYTES)
            .map(|index| index as u8 % 7)
            .collect::<Vec<_>>();
        info!("[Seq1] Origin length: {:?}", origin.len());

        let packets = FrameManager::<Ofdm>::construct(&origin);

        for packet in packets {
            if let Some(frame) = frame_manager.update(&packet) {
                info!("[Seq1] Frame length: {:?}", frame.len());
                assert_eq!(origin, frame);
            }
        }

        let origin = (0..packet_length - TEST_REDUCE_BYTES + 2)
            .map(|index| index as u8 % 8)
            .collect::<Vec<_>>();
        info!("[Seq2] Origin length: {:?}", origin.len());

        let packets = FrameManager::<Ofdm>::construct(&origin);

        for packet in packets {
            if let Some(frame) = frame_manager.update(&packet) {
                info!("[Seq2] Frame length: {:?}", frame.len());
                assert_eq!(origin, frame);
                return;
            }
        }

        unreachable!("Must be able to get frame response!");
    }
}
