use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
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
    pub fn create_buffer(size: usize) -> Self {
        let buffer = Vec::with_capacity(size);
        Self {
            inner: Arc::new(Mutex::new(AudioPacketVariant::Buffer(buffer))),
        }
    }

    pub fn create_reader(file: &'static str) -> Self {
        let reader = WavReader::open(file).unwrap();
        Self {
            inner: Arc::new(Mutex::new(AudioPacketVariant::Reader(reader))),
        }
    }

    pub fn create_writer(file: &'static str, sample_rate: u32) -> Self {
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
            AudioPacketVariant::Reader(reader) => match reader.samples::<i16>().nth(0) {
                Some(sample) => {
                    const AMPLITUDE: f32 = i16::MAX as f32;
                    Some(sample.unwrap() as f32 / AMPLITUDE)
                }
                None => None,
            },
            AudioPacketVariant::Writer(_) => panic!("Cannot read from writer!"),
        }
    }

    pub fn read_all(&self) -> Vec<f32> {
        let mut container = self.inner.lock().unwrap();
        match &mut *container {
            AudioPacketVariant::Buffer(buffer) => buffer.clone(),
            AudioPacketVariant::Reader(reader) => reader
                .samples::<i16>()
                .map(|sample| {
                    const AMPLITUDE: f32 = i16::MAX as f32;
                    sample.unwrap() as f32 / AMPLITUDE
                })
                .collect(),
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

    pub fn write_chunk(&self, chunk: &[f32]) {
        let mut container = self.inner.lock().unwrap();
        match &mut *container {
            AudioPacketVariant::Buffer(buffer) => buffer.extend_from_slice(chunk),
            AudioPacketVariant::Reader(_) => panic!("Cannot write to reader!"),
            AudioPacketVariant::Writer(writer) => chunk
                .iter()
                .for_each(|sample| writer.write_sample(*sample).unwrap()),
        }
    }
}
