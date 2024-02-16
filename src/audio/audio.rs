use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use jack::{AsyncClient, ClientOptions, Error};
use jack::{AudioIn, AudioOut, Client, Port};
use jack::{ClosureProcessHandler, Control, ProcessScope};

pub(crate) type AudioCallback = Box<dyn FnMut(&mut AudioPorts, &ProcessScope) + Send + Sync>;

type ClientCallback = impl Fn(&Client, &ProcessScope) -> Control + Send;
type AsyncClientCallback = AsyncClient<(), ClosureProcessHandler<ClientCallback>>;

const CLIENT_NAME_PREFIX: &str = "AcousticNetwork";

pub struct Audio {
    client: RefCell<Option<Client>>,
    ports: Mutex<Option<AudioPorts>>,
    pub timetick: AtomicUsize,
    active_client: RefCell<Option<AsyncClientCallback>>,
    pub sample_rate: Cell<Option<usize>>,
    callbacks: Mutex<Vec<AudioCallback>>,
}

pub struct AudioPorts {
    pub capture: Port<AudioIn>,
    pub playback: Port<AudioOut>,
}

pub enum AudioDeactivateFlag {
    Deactivate,
    Restart,
    CleanRestart,
}

impl Audio {
    pub fn new() -> Result<&'static Audio, Error> {
        let audio = Audio {
            client: RefCell::new(None),
            ports: Mutex::new(None),
            timetick: AtomicUsize::new(0),
            active_client: RefCell::new(None),
            sample_rate: Cell::new(None),
            callbacks: Mutex::new(Vec::new()),
        };

        audio.init_client()?;

        Ok(Box::leak(Box::new(audio)))
    }

    pub fn init_client(&self) -> Result<(), Error> {
        let random_node_id = rand::random::<u8>();
        let client_name = format!("{}-{}", CLIENT_NAME_PREFIX, random_node_id);

        let (client, _status) = Client::new(&client_name, ClientOptions::NO_START_SERVER)?;

        let in_port = client.register_port("input", AudioIn::default())?;
        let out_port = client.register_port("output", AudioOut::default())?;

        let sample_rate = client.sample_rate();
        *self.client.borrow_mut() = Some(client);

        let ports = AudioPorts {
            capture: in_port,
            playback: out_port,
        };

        *self.ports.lock().unwrap() = Some(ports);
        self.sample_rate.set(Some(sample_rate));
        self.timetick.store(0, Ordering::Relaxed);

        Ok(())
    }

    pub fn register(&'static self, callback: AudioCallback) {
        self.callbacks.lock().unwrap().push(callback);
    }

    pub fn activate(&'static self) {
        let Self {
            ports,
            timetick,
            callbacks,
            ..
        } = self;

        let client = self.client.borrow_mut().take();
        let buffer_size = client.as_ref().unwrap().buffer_size();

        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let mut ports = ports.lock().unwrap();

            let mut callbacks = callbacks.lock().unwrap();
            for callback in callbacks.iter_mut() {
                callback(&mut ports.as_mut().unwrap(), ps);
            }

            timetick.fetch_add(buffer_size as usize, Ordering::Relaxed);
            Control::Continue
        };

        let process = ClosureProcessHandler::new(process_callback);
        let active_client = client.unwrap().activate_async((), process).unwrap();

        {
            let client = active_client.as_client();

            let capture_port = client.port_by_name("system:capture_1").unwrap();
            let playback_port = client.port_by_name("system:playback_1").unwrap();

            if let Some(ports) = ports.lock().unwrap().as_ref() {
                client.connect_ports(&capture_port, &ports.capture).unwrap();
                client
                    .connect_ports(&ports.playback, &playback_port)
                    .unwrap();
            };
        }

        *self.active_client.borrow_mut() = Some(active_client);
    }

    pub fn deactivate(&self, flag: AudioDeactivateFlag) {
        let client = self.active_client.take().unwrap();
        client.deactivate().unwrap();

        match flag {
            AudioDeactivateFlag::Restart => {
                self.init_client().unwrap();
            }
            AudioDeactivateFlag::CleanRestart => {
                self.callbacks.lock().unwrap().clear();
                self.init_client().unwrap();
            }
            AudioDeactivateFlag::Deactivate => {}
        }
    }
}
