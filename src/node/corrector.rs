use reed_solomon::{Decoder, Encoder};

const ECC_LENGTH: usize = 96;
const DATA_LENGTH: usize = 255 - ECC_LENGTH;

pub struct ErrorCorrector;

impl ErrorCorrector {
    pub fn encode(data: &Vec<u8>) -> Vec<u8> {
        let encode = |chunk: &[u8]| {
            let encoder = Encoder::new(ECC_LENGTH);
            encoder.encode(&chunk).to_vec()
        };

        data.chunks(DATA_LENGTH).flat_map(encode).collect()
    }

    pub fn decode(data: &Vec<u8>) -> Vec<u8> {
        let decode = |chunk: &[u8]| {
            let mut chunk_mut = chunk.to_vec();
            chunk_mut.resize(255, 0);

            let result = Decoder::new(ECC_LENGTH)
                .correct(&mut chunk_mut, None)
                .unwrap_or_else(|_| {
                    error!("Cannot correct, too many errors!");
                    reed_solomon::Buffer::from_slice(&[], 0)
                });

            result.data().to_vec()
        };

        data.chunks(DATA_LENGTH + ECC_LENGTH)
            .flat_map(decode)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reed_solomon() {
        const TEST_SEQUENCE_BYTES: usize = 100;

        let data: Vec<_> = (0..TEST_SEQUENCE_BYTES)
            .map(|_| rand::random::<u8>())
            .collect();

        let mut encoded = ErrorCorrector::encode(&data);

        encoded[0] = 0;
        encoded[1] = 0;

        let mut decoded = ErrorCorrector::decode(&encoded);
        decoded.truncate(TEST_SEQUENCE_BYTES);

        assert_eq!(data, decoded);
    }
}
