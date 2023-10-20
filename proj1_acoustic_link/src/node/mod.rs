use once_cell::sync::Lazy;

mod receiver;
pub use receiver::Receiver;
mod sender;
pub use sender::Sender;

static WARMUP_SEQUENCE: Lazy<Vec<u8>> = Lazy::new(|| {
    const WARMUP_SEQUENCE_LENGTH: usize = 200;

    (0..WARMUP_SEQUENCE_LENGTH)
        .map(|_| rand::random::<u8>() % 2)
        .collect()
});
