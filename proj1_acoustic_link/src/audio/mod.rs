use std::cell::RefCell;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Mutex, RwLock};

use hound::{SampleFormat, WavSpec, WavWriter};
use jack::{AsyncClient, AudioIn, AudioOut, Error};
use jack::{Client, Port};
use jack::{ClosureProcessHandler, Control, ProcessScope};

pub type Callback = Box<dyn Fn(&mut AudioPorts, &ProcessScope) + Send + Sync>;
pub type ClientCallback = impl Fn(&Client, &ProcessScope) -> Control + Send;
pub type AsyncClientCallback = AsyncClient<(), ClosureProcessHandler<ClientCallback>>;

pub struct Audio {
    client: RefCell<Option<Client>>,
    pub ports: RwLock<AudioPorts>,
    pub timetick: Mutex<f32>,
    pub writer: Mutex<WavWriter<BufWriter<File>>>,
    active_client: RefCell<Option<AsyncClientCallback>>,
    pub sample_rate: RefCell<u32>,
    callbacks: RwLock<Vec<Callback>>,
}

pub struct AudioPorts {
    pub capture: Port<AudioIn>,
    pub playback: Port<AudioOut>,
}

impl Audio {
    pub fn init(name: &str) -> Result<&'static Audio, Error> {
        let (client, _status) =
            jack::Client::new(name, jack::ClientOptions::NO_START_SERVER)?;
    
        let in_port = client.register_port("input", jack::AudioIn::default())?;
        let out_port = client.register_port("output", jack::AudioOut::default())?;
        let capture_port = client.port_by_name("system:capture_1").unwrap();
        let playback_port = client.port_by_name("system:playback_1").unwrap();
        
        client.connect_ports(&capture_port, &in_port)?;
        client.connect_ports(&out_port, &playback_port)?;
        
        let ports = AudioPorts {
            capture: in_port,
            playback: out_port,
        };

        let sample_rate = client.sample_rate() as u32;

        let wav_spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let writer = WavWriter::create("outputs/output.wav", wav_spec).unwrap();

        let audio = Audio {
            client: RefCell::new(Some(client)),
            ports: RwLock::new(ports),
            timetick: Mutex::new(0.0),
            writer: Mutex::new(writer),
            active_client: RefCell::new(None),
            sample_rate: RefCell::new(sample_rate),
            callbacks: RwLock::new(Vec::new()),
        };

        Ok(Box::leak(Box::new(audio)))
    }

    pub fn register(&'static self, callback: Callback) {
        self.callbacks.write().unwrap().push(Box::new(callback));
    }

    pub fn activate(&'static self) {
        let Self { ports, callbacks, .. } = self;

        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let mut ports = ports.write().unwrap();
            let callbacks = callbacks.read().unwrap();
            callbacks.iter().for_each(|callback| callback(&mut ports, ps));
            Control::Continue
        };
        
        let process = ClosureProcessHandler::new(process_callback);

        let client = self.client.borrow_mut().take();
        let active_client = client.unwrap().activate_async((), process).unwrap();
        *self.active_client.borrow_mut() = Some(active_client);
    }

    pub fn deactivate(&self) {
        let mut active_client = self.active_client.borrow_mut();
        active_client.take().unwrap().deactivate().unwrap();
    }
}