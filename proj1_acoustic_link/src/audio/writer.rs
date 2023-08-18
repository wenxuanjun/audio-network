use hound::{SampleFormat, WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;

pub enum AudioWriterVariant {
    Buffer(Vec<f32>),
    File(WavWriter<BufWriter<File>>),
}

pub struct AudioWriter {
    inner: AudioWriterVariant,
}

impl AudioWriter {
    pub fn buffer(size: usize) -> Self {
        Self {
            inner: AudioWriterVariant::Buffer(vec![0.0; size]),
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
            inner: AudioWriterVariant::File(writer),
        }
    }

    pub fn write_sample(&mut self, sample: f32) {
        match &mut self.inner {
            AudioWriterVariant::Buffer(buffer) => buffer.push(sample),
            AudioWriterVariant::File(writer) => writer.write_sample(sample).unwrap(),
        }
    }
}
