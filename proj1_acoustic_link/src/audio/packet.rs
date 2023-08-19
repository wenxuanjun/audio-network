use hound::{SampleFormat, WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Mutex, Arc};

pub enum AudioPacketVariant {
    Buffer(Vec<f32>),
    File(WavWriter<BufWriter<File>>),
}

#[derive(Clone)]
pub struct AudioPacket {
    inner: Arc<Mutex<AudioPacketVariant>>,
}

impl AudioPacket {
    pub fn buffer(size: usize) -> Self {
        let buffer = Vec::with_capacity(size);
        Self {
            inner: Arc::new(Mutex::new(AudioPacketVariant::Buffer(buffer))),
        }
    }

    pub fn file(file: &'static str, sample_rate: u32) -> Self {
        let wav_spec = WavSpec {
            channels: 1,
            bits_per_sample: 32,
            sample_rate,
            sample_format: SampleFormat::Float,
        };
        let writer = WavWriter::create(file, wav_spec).unwrap();
        Self {
            inner: Arc::new(Mutex::new(AudioPacketVariant::File(writer))),
        }
    }

    pub fn read_sample(&self, index: usize) -> Option<f32> {
        let container = self.inner.lock().unwrap();
        match &*container {
            AudioPacketVariant::Buffer(buffer) => buffer.get(index).copied(),
            AudioPacketVariant::File(_) => panic!("Cannot read from file!"),
        }
    }

    pub fn write_sample(&self, sample: f32) {
        let mut container = self.inner.lock().unwrap();
        match &mut *container {
            AudioPacketVariant::Buffer(buffer) => buffer.push(sample),
            AudioPacketVariant::File(writer) => writer.write_sample(sample).unwrap(),
        }
    }
}
