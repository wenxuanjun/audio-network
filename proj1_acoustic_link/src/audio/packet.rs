use hound::{SampleFormat, WavSpec, WavWriter, WavReader};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};

pub enum AudioPacketVariant {
    Buffer(Vec<f32>),
    Reader(WavReader<BufReader<File>>),
    Writer(WavWriter<BufWriter<File>>),
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

    pub fn reader(file: &'static str) -> Self {
        let reader = WavReader::open(file).unwrap();
        Self {
            inner: Arc::new(Mutex::new(AudioPacketVariant::Reader(reader))),
        }
    }

    pub fn writer(file: &'static str, sample_rate: u32) -> Self {
        let wav_spec = WavSpec {
            channels: 1,
            bits_per_sample: 32,
            sample_rate,
            sample_format: SampleFormat::Float,
        };
        let writer = WavWriter::create(file, wav_spec).unwrap();
        Self {
            inner: Arc::new(Mutex::new(AudioPacketVariant::Writer(writer))),
        }
    }

    pub fn read_sample(&self, index: usize) -> Option<f32> {
        let mut container = self.inner.lock().unwrap();
        match &mut *container {
            AudioPacketVariant::Buffer(buffer) => buffer.get(index).copied(),
            AudioPacketVariant::Reader(reader) => {
                match reader.samples::<i16>().nth(0) {
                    Some(sample) => {
                        let amplitude = i16::MAX as f32;
                        Some(sample.unwrap() as f32 / amplitude)
                    },
                    None => None,
                }
            },
            AudioPacketVariant::Writer(_) => panic!("Cannot read from writer!"),
        }
    }

    pub fn write_sample(&self, sample: f32) {
        let mut container = self.inner.lock().unwrap();
        match &mut *container {
            AudioPacketVariant::Buffer(buffer) => buffer.push(sample),
            AudioPacketVariant::Reader(_) => panic!("Cannot write to reader!"),
            AudioPacketVariant::Writer(writer) => writer.write_sample(sample).unwrap(),
        }
    }
}
