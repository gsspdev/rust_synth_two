use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

struct Oscillator {
    frequency: f64,
    sample_rate: f64,
    phase: f64,
    phase_increment: f64,
}

impl Oscillator {
    fn new(freq: f64, rate: f64) -> Self {
        let phase_increment = 2.0 * std::f64::consts::PI * freq / rate;
        Oscillator {
            frequency: freq,
            sample_rate: rate,
            phase: 0.0,
            phase_increment,
        }
    }

    fn next_sample(&mut self) -> f64 {
        let sample = (self.phase).sin();
        self.phase += self.phase_increment;
        if self.phase >= 2.0 * std::f64::consts::PI {
            self.phase -= 2.0 * std::f64::consts::PI;
        }
        sample
    }
}

struct AudioEngine {
    oscillator: Arc<Mutex<Oscillator>>,
    stream: cpal::Stream,
}

impl AudioEngine {
    fn new(freq: f64, rate: f64) -> Self {
        let oscillator = Arc::new(Mutex::new(Oscillator::new(freq, rate)));
        let osc_clone = Arc::clone(&oscillator);

        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to find output device");
        let config = device.default_output_config().unwrap();

        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.config(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut oscillator = osc_clone.lock().unwrap();
                    for sample in data.iter_mut() {
                        *sample = oscillator.next_sample() as f32;
                    }
                },
                err_fn,
                None,
            ),
            // Add other sample formats if needed
            _ => panic!("Sample format not supported"),
        }.unwrap();

        stream.play().unwrap();

        AudioEngine { oscillator, stream }
    }
}

fn main() {
    let engine = AudioEngine::new(440.0, 44100.0);
    std::thread::sleep(std::time::Duration::from_secs(5)); // Play for 5 seconds
}
