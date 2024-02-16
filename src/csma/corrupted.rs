use crc::{Crc, CRC_16_USB};

pub(crate) const CRC_BYTES: usize = 2;
const CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_USB);

pub(crate) struct CrcWrapper;

impl CrcWrapper {
    pub fn encode(data: &[u8]) -> Vec<u8> {
        let crc = CRC16.checksum(&data);
        let mut result = data.to_vec();
        result.extend_from_slice(&crc.to_be_bytes());
        result
    }

    pub fn decode(data: &[u8]) -> Option<Vec<u8>> {
        let (data, checksum) = data.split_at(data.len() - CRC_BYTES);
        let crc = CRC16.checksum(&data);
        if crc == u16::from_be_bytes([checksum[0], checksum[1]]) {
            Some(data.to_vec())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SEQUENCE_BYTES: usize = 1920;

    #[test]
    fn test_crc_wrapper() {
        let data = (0..TEST_SEQUENCE_BYTES)
            .map(|_| rand::random::<u8>())
            .collect::<Vec<_>>();

        let encoded = CrcWrapper::encode(&data);
        let decoded = CrcWrapper::decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_crc_wrapper_corrupted() {
        let data = (0..TEST_SEQUENCE_BYTES)
            .map(|_| rand::random::<u8>())
            .collect::<Vec<_>>();

        let mut encoded = CrcWrapper::encode(&data);
        encoded[0] = encoded[0].wrapping_add(1);
        let decoded = CrcWrapper::decode(&encoded);
        assert!(decoded.is_none());
    }
}
