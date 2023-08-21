use std::cell::RefCell;
use std::sync::RwLock;

use jack::{AsyncClient, ClientOptions, Error, LatencyType};
use jack::{AudioIn, AudioOut, Client, Port};
use jack::{ClosureProcessHandler, Control, ProcessScope};

pub type Callback = Box<dyn Fn(&mut AudioPorts, &ProcessScope) + Send + Sync>;
type ClientCallback = impl Fn(&Client, &ProcessScope) -> Control + Send;
type AsyncClientCallback = AsyncClient<(), ClosureProcessHandler<ClientCallback>>;

const CLIENT_NAME: &str = "AcousticNetwork";

pub struct Audio {
    client: RefCell<Option<Client>>,
    ports: RwLock<Option<AudioPorts>>,
    pub timetick: RwLock<u64>,
    active_client: RefCell<Option<AsyncClientCallback>>,
    pub sample_rate: RefCell<Option<usize>>,
    callbacks: RwLock<Vec<Callback>>,
}

pub struct AudioPorts {
    pub capture: Port<AudioIn>,
    pub playback: Port<AudioOut>,
}

pub enum AudioDeactivateFlags {
    Deactivate,
    Restart,
    CleanRestart,
}

impl Audio {
    pub fn new() -> Result<&'static Audio, Error> {
        let audio = Audio {
            client: RefCell::new(None),
            ports: RwLock::new(None),
            timetick: RwLock::new(0),
            active_client: RefCell::new(None),
            sample_rate: RefCell::new(None),
            callbacks: RwLock::new(Vec::new()),
        };

        Ok(Box::leak(Box::new(audio)))
    }

    pub fn init_client(&self) -> Result<(), Error> {
        let (client, _status) =
            Client::new(CLIENT_NAME, ClientOptions::NO_START_SERVER)?;
    
        let capture_port = client.port_by_name("system:capture_1").unwrap();
        let playback_port = client.port_by_name("system:playback_1").unwrap();

        let in_port = client.register_port("input", AudioIn::default())?;
        let out_port = client.register_port("output", AudioOut::default())?;
        client.connect_ports(&capture_port, &in_port)?;
        client.connect_ports(&out_port, &playback_port)?;

        let sample_rate = client.sample_rate();
        *self.client.borrow_mut() = Some(client);

        let ports = AudioPorts {
            capture: in_port,
            playback: out_port,
        };

        *self.ports.write().unwrap() = Some(ports);
        *self.sample_rate.borrow_mut() = Some(sample_rate);
        *self.timetick.write().unwrap() = 0;

        Ok(())
    }

    pub fn register(&'static self, callback: Callback) {
        self.callbacks.write().unwrap().push(callback);
    }

    pub fn activate(&'static self) {
        let Self { ports, timetick, callbacks, .. } = self;

        let client = self.client.borrow_mut().take();
        let buffer_size = client.as_ref().unwrap().buffer_size();

        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let mut ports = ports.write().unwrap();

            let callbacks = callbacks.read().unwrap();
            callbacks.iter().for_each(|callback| callback(&mut ports.as_mut().unwrap(), ps));

            *timetick.write().unwrap() += buffer_size as u64;
            Control::Continue
        };
        
        let process = ClosureProcessHandler::new(process_callback);

        let active_client = client.unwrap().activate_async((), process).unwrap();
        *self.active_client.borrow_mut() = Some(active_client);
    }

    pub fn deactivate(&self, flags: AudioDeactivateFlags) {
        let client = self.active_client.take().unwrap();
        client.deactivate().unwrap();
    
        match flags {
            AudioDeactivateFlags::Restart => {
                self.init_client().unwrap();
            },
            AudioDeactivateFlags::CleanRestart => {
                self.clear_callbacks();
                self.init_client().unwrap();
            },
            AudioDeactivateFlags::Deactivate => {}
        }
    }    

    pub fn clear_callbacks(&self) {
        self.callbacks.write().unwrap().clear();
    }

    pub fn get_latency(&self) -> f64 {
        let min_latency = {
            let ports = self.ports.read().unwrap();
            ports.as_ref().unwrap().capture.get_latency_range(LatencyType::Capture).0
        };
        let sample_rate = self.sample_rate.borrow().unwrap();
        min_latency as f64 / sample_rate as f64 * 1000.0
    }
}
