use crate::number::FP;

mod psk;
pub use psk::Psk;

mod ofdm;
pub use ofdm::Ofdm;

pub trait Modem {
    const PREFERED_PAYLOAD_BYTES: usize;

    fn new(sample_rate: usize) -> Self;
    fn modulate(&self, bytes: &Vec<u8>) -> Vec<FP>;
    fn demodulate(&self, samples: &Vec<FP>) -> Vec<u8>;
}

pub struct BitByteConverter;

impl BitByteConverter {
    pub fn bytes_to_bits(bytes: &[u8]) -> Vec<u8> {
        let mut bits = Vec::new();
        for byte in bytes {
            for i in 0..8 {
                bits.push((byte >> i) & 0x01);
            }
        }
        bits
    }

    pub fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for chunk in bits.chunks(8) {
            let mut byte = 0;
            for (i, bit) in chunk.iter().enumerate() {
                byte |= (*bit as u8) << i;
            }
            bytes.push(byte);
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_byte_converter() {
        let bytes = vec![0x01, 0x02];
        let bits = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];

        let short_bits = vec![1, 0, 1, 1];
        let short_bytes = vec![0x0D];

        assert_eq!(BitByteConverter::bytes_to_bits(&bytes), bits);
        assert_eq!(BitByteConverter::bits_to_bytes(&bits), bytes);
        assert_eq!(BitByteConverter::bits_to_bytes(&short_bits), short_bytes);
    }
}
