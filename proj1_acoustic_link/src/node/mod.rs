use once_cell::sync::Lazy;

mod corrector;
pub use corrector::ErrorCorrector;

mod frame_manager;
pub use frame_manager::FrameManager;

mod receiver;
pub use receiver::{Receiver, AveragePower};

mod sender;
pub use sender::Sender;

static WARMUP_SEQUENCE: Lazy<Vec<u8>> = Lazy::new(|| {
    #[cfg(feature = "cable_link")]
    const WARMUP_SEQUENCE_BYTES: usize = 0;
    #[cfg(not(feature = "cable_link"))]
    const WARMUP_SEQUENCE_BYTES: usize = 24;

    (0..WARMUP_SEQUENCE_BYTES)
        .map(|_| rand::random::<u8>())
        .collect()
});
