use jack::ProcessScope;
use proj1_acoustic_link::audio::{Audio, AudioPorts};

#[test]
fn part1_ck1() {
    let audio = Audio::new().unwrap();

    let sample_rate = audio.sample_rate;
    let audio_output = AudioWriter::file("shit", sample_rate as u32);
    audio.init_writer(audio_output);

    println!("Latency of port: {:.2} ms", audio.get_latency());

    let Audio { timetick, writer, .. } = audio;

    let capture_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
        for sample in ports.capture.as_slice(&ps).iter() {
            let mut writer = writer.lock().unwrap();
            writer.as_mut().unwrap().write_sample(*sample);
        }
    };

    let playback_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
        for sample in ports.playback.as_mut_slice(&ps).iter_mut() {
            let mut time = timetick.lock().unwrap();
            let multiplier = 2.0 * std::f32::consts::PI * *time;
            *sample = (multiplier * 1000.0).sin() + (multiplier * 15000.0).sin();
            *time += 1.0 / sample_rate as f32;
        }
    };

    audio.register(Box::new(capture_callback));
    audio.register(Box::new(playback_callback));

    audio.activate();

    println!("Press enter to end recording...");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input).ok();

    println!("Deactivating client...");
    audio.deactivate();
}
