mod psk;
pub use psk::PSK;

pub trait Modem {
    const BIT_RATE: usize;
    const CARRIER_FREQUENCY: f32;

    fn modulate(&self, bytes: &Vec<u8>) -> Vec<f32>;
    fn demodulate(&self, samples: &Vec<f32>) -> Vec<u8>;
}
