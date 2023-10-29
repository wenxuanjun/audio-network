use reed_solomon::{Decoder, Encoder};

const ECC_LENGTH: usize = 32;
const DATA_LENGTH: usize = 255 - ECC_LENGTH;

pub struct ErrorCorrector;

impl ErrorCorrector {
    pub fn encode(data: &Vec<u8>) -> Vec<u8> {
        data.chunks(DATA_LENGTH)
            .flat_map(|chunk| Encoder::new(ECC_LENGTH).encode(&chunk).to_vec())
            .collect()
    }

    pub fn decode(data: &Vec<u8>) -> Vec<u8> {
        let decode = |chunk: &[u8]| {
            let mut chunk_mut = chunk.to_vec();
            chunk_mut.resize(255, 0);

            Decoder::new(ECC_LENGTH)
                .correct(&mut chunk_mut, None)
                .unwrap_or_else(|_| {
                    println!("Cannot correct, too many errors!");
                    std::process::exit(1)
                })
                .data()
                .to_vec()
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
        const TEST_SEQUENCE_BYTES: usize = 1000;

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
