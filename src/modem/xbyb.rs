use super::Modem;
use crate::number::FP;

use bitvec::prelude::*;
type BitVecU8 = BitVec<u8, Msb0>;

const SAMPLE_REPEAT_TIMES: usize = 2;
const BYTES_PER_PACKET: usize = 100;
const BITS_PER_PACKET: usize = BYTES_PER_PACKET * 8;
const SAMPLES_PER_PACKET: usize = (SAMPLE_REPEAT_TIMES * BITS_PER_PACKET) / 4 * 5;

pub struct BitWave;

impl Modem for BitWave {
    const MIN_MODULATE_BYTES: usize = BYTES_PER_PACKET;
    const PREFERED_PAYLOAD_BYTES: usize = BYTES_PER_PACKET;
    const PREAMBLE_FREQUENCY_RANGE: (f32, f32) = (900.0, 3000.0);

    fn new(_: usize) -> Self {
        Self
    }

    fn modulate(&self, bytes: &[u8]) -> Vec<FP> {
        bytes
            .chunks(BYTES_PER_PACKET)
            .flat_map(|chunk| {
                let bit_vec = BitVec::from_slice(chunk);

                Self::encode_nrzi(Self::encode_4b5b(bit_vec))
                    .into_iter()
                    .flat_map(|x| [FP::from(if x { 1.0 } else { -1.0 }); SAMPLE_REPEAT_TIMES])
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn demodulate(&self, samples: &[FP]) -> Vec<u8> {
        samples
            .chunks(SAMPLES_PER_PACKET)
            .flat_map(|chunk| {
                let code_bits = chunk
                    .chunks_exact(SAMPLE_REPEAT_TIMES)
                    .into_iter()
                    .map(|x| x.iter().fold(FP::ZERO, |acc, &x| acc + x) > FP::ZERO)
                    .collect();

                Self::decode_4b5b(Self::decode_nrzi(code_bits)).into_vec()
            })
            .collect::<Vec<_>>()
    }
}

impl BitWave {
    const B5B_TABLE: [u8; 16] = [
        0b_11110, 0b_01001, 0b_10100, 0b_10101, 0b_01010, 0b_01011, 0b_01110, 0b_01111, 0b_10010,
        0b_10011, 0b_10110, 0b_10111, 0b_11010, 0b_11011, 0b_11100, 0b_11101,
    ];

    fn encode_nrzi(bits: BitVecU8) -> BitVecU8 {
        let mut current = false;
        bits.iter()
            .map(|bit| {
                current = current != *bit;
                current
            })
            .collect()
    }

    fn decode_nrzi(bits: BitVecU8) -> BitVecU8 {
        let mut current = false;
        bits.iter()
            .map(|bit| {
                let result = current != *bit;
                current = *bit;
                result
            })
            .collect()
    }

    fn encode_4b5b(bits: BitVecU8) -> BitVecU8 {
        assert!(bits.len() % 4 == 0);
        let mut out = BitVecU8::with_capacity(bits.len() / 4 * 5);
        bits.chunks_exact(4).for_each(|bits| {
            let val = bits.load_be::<usize>();
            out.extend(&BitVecU8::from_element(Self::B5B_TABLE[val])[3..]);
        });
        out
    }

    fn decode_4b5b(bits: BitVecU8) -> BitVecU8 {
        assert!(bits.len() % 5 == 0);
        let mut out = BitVecU8::with_capacity(bits.len() / 5 * 4);
        bits.chunks_exact(5).for_each(|bits| {
            let val_4b = Self::B5B_TABLE
                .iter()
                .position(|&map_5b| map_5b == bits.load_be::<u8>())
                .unwrap_or(0) as u8;
            out.extend(&BitVecU8::from_element(val_4b)[4..]);
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SEQUENCE_BYTES: usize = 100;

    #[test]
    fn test_bitwave() {
        let data = (0..TEST_SEQUENCE_BYTES)
            .map(|_| rand::random::<u8>())
            .collect::<Vec<_>>();

        let bitwave = BitWave::new(0);

        let mut modulated = bitwave.modulate(&data);

        println!("Modulated data samples: {:?}", modulated.len());

        modulated
            .iter_mut()
            .for_each(|sample| *sample += FP::from(rand::random::<f32>()) / FP::from(2.0));

        let demodulated = bitwave.demodulate(&modulated);

        assert_eq!(data, demodulated);
    }
}
