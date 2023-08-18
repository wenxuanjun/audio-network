use jack::{LatencyType, ProcessScope};
use proj1_acoustic_link::audio::{Audio, AudioPorts};

fn main() {
    let audio = Audio::init("AcousticLink").unwrap();
    let sample_rate = *audio.sample_rate.borrow();

    let min_latency = {
        let ports = audio.ports.read().unwrap();
        ports.capture.get_latency_range(LatencyType::Capture).0
    };

    let latency = min_latency as f64 / sample_rate as f64 * 1000.0;
    println!("Latency of port: {:.2} ms", latency);

    let Audio { timetick, writer, .. } = audio;

    let callback1 = move | ports: &mut AudioPorts, ps: &ProcessScope| {
        for sample in ports.capture.as_slice(&ps).iter() {
            writer.lock().unwrap().write_sample(*sample).unwrap();
        }
    };

    let callback2 = move | ports: &mut AudioPorts, ps: &ProcessScope| {
        for sample in ports.playback.as_mut_slice(&ps).iter_mut() {
            let mut time = timetick.lock().unwrap();
            let multiplier = 2.0 * std::f32::consts::PI * *time;
            *sample = (multiplier * 1000.0).sin() + (multiplier * 15000.0).sin();
            *time += 1.0 / sample_rate as f32;
        }
    };

    audio.register(Box::new(callback1));
    audio.register(Box::new(callback2));

    audio.activate();

    println!("Press enter to end recording...");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input).ok();

    println!("Deactivating client...");
    audio.deactivate();
}
