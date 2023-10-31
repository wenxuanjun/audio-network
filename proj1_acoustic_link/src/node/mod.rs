use once_cell::sync::Lazy;

mod receiver;
pub use receiver::Receiver;
mod sender;
pub use sender::Sender;
mod corrector;
pub use corrector::ErrorCorrector;

static WARMUP_SEQUENCE: Lazy<Vec<u8>> = Lazy::new(|| {
    const WARMUP_SEQUENCE_BYTES: usize = 24;

    (0..WARMUP_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect()
});
