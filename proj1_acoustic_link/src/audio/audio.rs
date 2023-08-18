use std::cell::RefCell;
use std::sync::{Mutex, RwLock};

use jack::{AsyncClient, ClientOptions, Error, LatencyType};
use jack::{AudioIn, AudioOut, Client, Port};
use jack::{ClosureProcessHandler, Control, ProcessScope};

use super::writer::AudioWriter;

pub type Callback = Box<dyn Fn(&mut AudioPorts, &ProcessScope) + Send + Sync>;
type ClientCallback = impl Fn(&Client, &ProcessScope) -> Control + Send;
type AsyncClientCallback = AsyncClient<(), ClosureProcessHandler<ClientCallback>>;

const CLIENT_NAME: &str = "AcousticLink";

pub struct Audio {
    client: RefCell<Option<Client>>,
    ports: RwLock<AudioPorts>,
    pub sample_rate: usize,
    pub timetick: Mutex<f32>,
    pub writer: Mutex<Option<AudioWriter>>,
    active_client: RefCell<Option<AsyncClientCallback>>,
    callbacks: RwLock<Vec<Callback>>,
}

pub struct AudioPorts {
    pub capture: Port<AudioIn>,
    pub playback: Port<AudioOut>,
}

impl Audio {
    pub fn new() -> Result<&'static Audio, Error> {
        let (client, _status) =
            jack::Client::new(CLIENT_NAME, ClientOptions::NO_START_SERVER)?;
    
        let capture_port = client.port_by_name("system:capture_1").unwrap();
        let playback_port = client.port_by_name("system:playback_1").unwrap();

        let in_port = client.register_port("input", AudioIn::default())?;
        let out_port = client.register_port("output", AudioOut::default())?;
        client.connect_ports(&capture_port, &in_port)?;
        client.connect_ports(&out_port, &playback_port)?;
        
        let ports = AudioPorts {
            capture: in_port,
            playback: out_port,
        };

        let sample_rate = client.sample_rate();

        let audio = Audio {
            client: RefCell::new(Some(client)),
            ports: RwLock::new(ports),
            timetick: Mutex::new(0.0),
            writer: Mutex::new(None),
            active_client: RefCell::new(None),
            callbacks: RwLock::new(Vec::new()),
            sample_rate,
        };

        Ok(Box::leak(Box::new(audio)))
    }

    pub fn init_writer(&self, writer: AudioWriter) {
        *self.writer.lock().unwrap() = Some(writer);
    }

    pub fn register(&'static self, callback: Callback) {
        self.callbacks.write().unwrap().push(callback);
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

    pub fn get_latency(&self) -> f64 {
        let min_latency = {
            let ports = self.ports.read().unwrap();
            ports.capture.get_latency_range(LatencyType::Capture).0
        };
        min_latency as f64 / self.sample_rate as f64 * 1000.0
    }
}
