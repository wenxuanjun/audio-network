mod psk;
pub use psk::PSK;

pub trait Modem {
    const BIT_RATE: usize;
    const CARRIER_FREQUENCY: f32;

    fn modulate(&self, bytes: &Vec<u8>) -> Vec<f32>;
    fn demodulate(&self, samples: &Vec<f32>) -> Vec<u8>;
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

mod tests {
    use super::*;

    #[test]
    fn test_bit_byte_converter() {
        let bytes = vec![0x01, 0x02, 0x03, 0x04];
        let bits = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
            0, 0, 0,
        ];

        assert_eq!(BitByteConverter::bytes_to_bits(&bytes), bits);
        assert_eq!(BitByteConverter::bits_to_bytes(&bits), bytes);
    }
}