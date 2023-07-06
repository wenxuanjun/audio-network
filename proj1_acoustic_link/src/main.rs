fn main() {
    let (client, _status) =
        jack::Client::new("AcousticLink", jack::ClientOptions::NO_START_SERVER).unwrap();
    
    let in_port = client.register_port("input", jack::AudioIn::default()).unwrap();
    let mut out_port = client.register_port("output", jack::AudioOut::default()).unwrap();
    let capture_port = client.port_by_name("system:capture_1").unwrap();
    let playback_port = client.port_by_name("system:playback_1").unwrap();
    
    client.connect_ports(&capture_port, &in_port).unwrap();
    client.connect_ports(&out_port, &playback_port).unwrap();

    let sample_rate = client.sample_rate() as u32;

    let min_latency = in_port.get_latency_range(jack::LatencyType::Capture).0 as f64;
    println!("Latency of port: {:.2} ms", min_latency / sample_rate as f64 * 1000.0);
    
    let wav_spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    
    let mut writer = hound::WavWriter::create("output.wav", wav_spec).unwrap();

    let mut time = 0.0;

    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        for sample in in_port.as_slice(ps).iter() {
            writer.write_sample(*sample).unwrap();
        }
        for sample in out_port.as_mut_slice(ps).iter_mut() {
            let multiplier = 2.0 * std::f32::consts::PI * time;
            *sample = (multiplier * 1000.0).sin() + (multiplier * 10000.0).sin();
            time += 1.0 / sample_rate as f32;
        }
        jack::Control::Continue
    };
    
    let process = jack::ClosureProcessHandler::new(process_callback);
    let active_client = client.activate_async((), process).unwrap();

    println!("Press enter to end recording...");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}
