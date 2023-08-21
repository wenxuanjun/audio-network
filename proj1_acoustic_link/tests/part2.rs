use jack::ProcessScope;
use proj1_acoustic_link::audio::AudioDeactivateFlags;
use proj1_acoustic_link::audio::{Audio, AudioPorts};

#[test]
fn part2_ck1() {
    let audio = Audio::new().unwrap();
    audio.init_client().unwrap();

    let timetick = &audio.timetick;
    let sample_rate = audio.sample_rate.borrow().unwrap();

    let sine_wave_callback = move |ports: &mut AudioPorts, ps: &ProcessScope| {
        let time = *timetick.read().unwrap() as f32;
        let buffer = ports.playback.as_mut_slice(&ps);
        for (index, sample) in buffer.iter_mut().enumerate() {
            let current_time = (index as f32 + time) / sample_rate as f32;
            let multiplier = 2.0 * std::f32::consts::PI * current_time;
            *sample = ((multiplier * 1000.0).sin() + (multiplier * 10000.0).sin()) / 2.0;
        }
    };

    audio.register(Box::new(sine_wave_callback));
    audio.activate();

    println!("Press enter to stop generating sine wave...");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input).ok();

    println!("Deactivating client...");
    audio.deactivate(AudioDeactivateFlags::Deactivate);
}
