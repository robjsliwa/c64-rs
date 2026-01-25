use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::{Arc, Mutex};

/// Audio output handler that generates samples from SID
pub struct SidAudioCallback {
    sid: Arc<Mutex<SidState>>,
}

/// Shared state between main thread and audio callback
pub struct SidState {
    samples: Vec<i16>,
    read_pos: usize,
}

impl SidState {
    pub fn new() -> Self {
        Self {
            samples: Vec::with_capacity(4096),
            read_pos: 0,
        }
    }

    pub fn push_sample(&mut self, sample: i16) {
        self.samples.push(sample);
    }

    pub fn clear_old_samples(&mut self) {
        // Keep buffer from growing unbounded
        if self.samples.len() > 8192 {
            self.samples.drain(0..self.read_pos);
            self.read_pos = 0;
        }
    }
}

impl AudioCallback for SidAudioCallback {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        let mut state = self.sid.lock().unwrap();

        for sample in out.iter_mut() {
            if state.read_pos < state.samples.len() {
                *sample = state.samples[state.read_pos];
                state.read_pos += 1;
            } else {
                *sample = 0; // Silence if no samples available
            }
        }
    }
}

/// Initialize SDL audio and return device + shared state
pub fn init_audio(
    sdl_context: &sdl2::Sdl,
) -> Result<(AudioDevice<SidAudioCallback>, Arc<Mutex<SidState>>), String> {
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // Mono
        samples: Some(512), // Small buffer for low latency
    };

    let sid_state = Arc::new(Mutex::new(SidState::new()));
    let sid_state_clone = Arc::clone(&sid_state);

    let audio_device =
        audio_subsystem.open_playback(None, &desired_spec, |_spec| SidAudioCallback {
            sid: sid_state_clone,
        })?;

    // Start playback
    audio_device.resume();

    Ok((audio_device, sid_state))
}
