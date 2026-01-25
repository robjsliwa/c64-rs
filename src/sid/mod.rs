// SID (Sound Interface Device) chip emulation
// Implementation of the 6581/8580 sound chip

mod envelope;
mod filter;
mod oscillator;
mod voice;

pub use envelope::Envelope;
pub use filter::{Filter, FilterMode};
pub use oscillator::{Oscillator, Waveform};
pub use voice::Voice;

/// SID chip with 3 voices, filter, and volume control
pub struct Sid {
    // Three independent voices
    voices: [Voice; 3],

    // Filter parameters
    filter_cutoff: u16,   // 11-bit (registers $D415-$D416)
    filter_resonance: u8, // 4-bit
    filter_routing: u8,   // Which voices route through filter (bits 0-2)
    filter_mode: FilterMode,
    filter: Filter,

    // Global volume
    volume: u8, // 4-bit master volume (0-15)

    // Sampling
    sample_rate: u32,
    cycles_per_sample: f64,
    cycle_counter: f64,

    // Last written values for read-back
    register_cache: [u8; 32],
}

impl Sid {
    /// Create a new SID chip instance
    pub fn new(sample_rate: u32) -> Self {
        let cycles_per_sample = 1_000_000.0 / sample_rate as f64; // ~1MHz clock / sample rate

        Self {
            voices: [Voice::new(0), Voice::new(1), Voice::new(2)],
            filter_cutoff: 0,
            filter_resonance: 0,
            filter_routing: 0,
            filter_mode: FilterMode::Off,
            filter: Filter::new(),
            volume: 0,
            sample_rate,
            cycles_per_sample,
            cycle_counter: 0.0,
            register_cache: [0; 32],
        }
    }

    /// Write to a SID register
    pub fn write(&mut self, reg: u8, value: u8) {
        let reg = reg & 0x1F; // Only 32 registers
        self.register_cache[reg as usize] = value;

        match reg {
            // Voice 1 (0x00-0x06)
            0x00 => self.voices[0].set_freq_lo(value),
            0x01 => self.voices[0].set_freq_hi(value),
            0x02 => self.voices[0].set_pw_lo(value),
            0x03 => self.voices[0].set_pw_hi(value),
            0x04 => self.voices[0].set_control(value),
            0x05 => self.voices[0].set_attack_decay(value),
            0x06 => self.voices[0].set_sustain_release(value),

            // Voice 2 (0x07-0x0D)
            0x07 => self.voices[1].set_freq_lo(value),
            0x08 => self.voices[1].set_freq_hi(value),
            0x09 => self.voices[1].set_pw_lo(value),
            0x0A => self.voices[1].set_pw_hi(value),
            0x0B => self.voices[1].set_control(value),
            0x0C => self.voices[1].set_attack_decay(value),
            0x0D => self.voices[1].set_sustain_release(value),

            // Voice 3 (0x0E-0x14)
            0x0E => self.voices[2].set_freq_lo(value),
            0x0F => self.voices[2].set_freq_hi(value),
            0x10 => self.voices[2].set_pw_lo(value),
            0x11 => self.voices[2].set_pw_hi(value),
            0x12 => self.voices[2].set_control(value),
            0x13 => self.voices[2].set_attack_decay(value),
            0x14 => self.voices[2].set_sustain_release(value),

            // Filter cutoff low (0x15)
            0x15 => {
                self.filter_cutoff = (self.filter_cutoff & 0x7F8) | ((value & 0x07) as u16);
                self.filter.set_cutoff(self.filter_cutoff);
            }

            // Filter cutoff high (0x16)
            0x16 => {
                self.filter_cutoff = (self.filter_cutoff & 0x007) | ((value as u16) << 3);
                self.filter.set_cutoff(self.filter_cutoff);
            }

            // Filter resonance and routing (0x17)
            0x17 => {
                self.filter_resonance = value >> 4;
                self.filter_routing = value & 0x0F;
                self.filter.set_resonance(self.filter_resonance);
            }

            // Filter mode and volume (0x18)
            0x18 => {
                self.filter_mode = FilterMode::from_bits(value >> 4);
                self.filter.set_mode(self.filter_mode);
                self.volume = value & 0x0F;
            }

            _ => {} // Other registers not used or write-only
        }
    }

    /// Read from a SID register
    pub fn read(&self, reg: u8) -> u8 {
        let reg = reg & 0x1F;
        match reg {
            // OSC3 output - voice 3 oscillator (random waveform read)
            0x1B => self.voices[2].oscillator_output(),
            // ENV3 output - voice 3 envelope
            0x1C => self.voices[2].envelope_output(),
            // Most registers are write-only, return last written value
            _ => self.register_cache[reg as usize],
        }
    }

    /// Clock the SID by one cycle
    pub fn clock(&mut self) {
        // Get zero-crossing information for sync
        let zero_crossings = [
            self.voices[0].crossed_zero(),
            self.voices[1].crossed_zero(),
            self.voices[2].crossed_zero(),
        ];

        // Clock each voice with sync sources
        // Voice 1 synced by voice 3
        self.voices[0].clock(zero_crossings[2]);

        // Voice 2 synced by voice 1
        self.voices[1].clock(zero_crossings[0]);

        // Voice 3 synced by voice 2
        self.voices[2].clock(zero_crossings[1]);

        self.cycle_counter += 1.0;
    }

    /// Check if a sample should be generated
    pub fn should_generate_sample(&self) -> bool {
        self.cycle_counter >= self.cycles_per_sample
    }

    /// Generate an audio sample
    pub fn generate_sample(&mut self) -> i16 {
        self.cycle_counter -= self.cycles_per_sample;

        let mut output = 0i32;
        let mut filtered_output = 0.0f32;
        let mut unfiltered_output = 0i32;

        // Mix voices with ring modulation
        // Voice 1 ring mods with voice 3
        let voice0_sample = if self.voices[0].ring_mod {
            self.voices[0].output_with_ring_mod(&self.voices[2])
        } else {
            self.voices[0].output()
        };

        // Voice 2 ring mods with voice 1
        let voice1_sample = if self.voices[1].ring_mod {
            self.voices[1].output_with_ring_mod(&self.voices[0])
        } else {
            self.voices[1].output()
        };

        // Voice 3 ring mods with voice 2
        let voice2_sample = if self.voices[2].ring_mod {
            self.voices[2].output_with_ring_mod(&self.voices[1])
        } else {
            self.voices[2].output()
        };

        let voice_samples = [voice0_sample, voice1_sample, voice2_sample];

        for (i, &sample) in voice_samples.iter().enumerate() {
            // Check if this voice routes through filter
            if self.filter_routing & (1 << i) != 0 {
                filtered_output += sample as f32;
            } else {
                unfiltered_output += sample as i32;
            }
        }

        // Process filter
        if self.filter_mode != FilterMode::Off {
            filtered_output = self.filter.process(filtered_output);
        }

        // Combine filtered and unfiltered
        output = unfiltered_output + filtered_output as i32;

        // Apply master volume (4-bit: 0-15)
        output = (output * self.volume as i32) / 15;

        // Clamp to i16 range
        output.clamp(-32768, 32767) as i16
    }

    /// Reset the SID chip
    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
        self.filter_cutoff = 0;
        self.filter_resonance = 0;
        self.filter_routing = 0;
        self.filter_mode = FilterMode::Off;
        self.volume = 0;
        self.cycle_counter = 0.0;
        self.register_cache = [0; 32];
        self.filter.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sid_new() {
        let sid = Sid::new(44100);
        assert_eq!(sid.volume, 0);
        assert_eq!(sid.filter_routing, 0);
    }

    #[test]
    fn test_sid_volume_write() {
        let mut sid = Sid::new(44100);
        sid.write(0x18, 0x0F); // Max volume
        assert_eq!(sid.volume, 15);
    }

    #[test]
    fn test_sid_filter_cutoff() {
        let mut sid = Sid::new(44100);
        sid.write(0x15, 0x07); // Low 3 bits
        sid.write(0x16, 0xFF); // High 8 bits
        assert_eq!(sid.filter_cutoff, 0x7FF); // 11 bits total
    }

    #[test]
    fn test_sid_voice_frequency() {
        let mut sid = Sid::new(44100);
        sid.write(0x00, 0x34); // Voice 1 freq lo
        sid.write(0x01, 0x12); // Voice 1 freq hi
        assert_eq!(sid.voices[0].frequency(), 0x1234);
    }

    #[test]
    fn test_sid_read_osc3() {
        let sid = Sid::new(44100);
        // Reading OSC3 should return oscillator output
        let _val = sid.read(0x1B);
        // Value depends on oscillator state, just verify it doesn't crash
    }

    #[test]
    fn test_sid_read_env3() {
        let sid = Sid::new(44100);
        // Reading ENV3 should return envelope output
        let _val = sid.read(0x1C);
        // Value depends on envelope state, just verify it doesn't crash
    }
}
