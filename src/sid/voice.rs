use crate::sid::{Envelope, Oscillator, Waveform};

/// A single SID voice with oscillator and envelope
pub struct Voice {
    id: usize,                    // Voice number (0-2)
    pub oscillator: Oscillator,
    pub envelope: Envelope,
    
    // Control register state
    frequency: u16,               // 16-bit frequency
    pulse_width: u16,             // 12-bit pulse width
    waveform: Waveform,
    gate: bool,
    sync: bool,                   // Oscillator sync enable
    pub ring_mod: bool,           // Ring modulation enable
    test: bool,                   // Test bit
}

impl Voice {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            oscillator: Oscillator::new(),
            envelope: Envelope::new(),
            frequency: 0,
            pulse_width: 0,
            waveform: Waveform::None,
            gate: false,
            sync: false,
            ring_mod: false,
            test: false,
        }
    }

    /// Set frequency low byte
    pub fn set_freq_lo(&mut self, value: u8) {
        self.frequency = (self.frequency & 0xFF00) | (value as u16);
        self.oscillator.set_frequency(self.frequency);
    }

    /// Set frequency high byte
    pub fn set_freq_hi(&mut self, value: u8) {
        self.frequency = (self.frequency & 0x00FF) | ((value as u16) << 8);
        self.oscillator.set_frequency(self.frequency);
    }

    /// Get current frequency
    pub fn frequency(&self) -> u16 {
        self.frequency
    }

    /// Set pulse width low byte (low 8 bits of 12-bit value)
    pub fn set_pw_lo(&mut self, value: u8) {
        self.pulse_width = (self.pulse_width & 0x0F00) | (value as u16);
        self.oscillator.set_pulse_width(self.pulse_width);
    }

    /// Set pulse width high byte (high 4 bits of 12-bit value)
    pub fn set_pw_hi(&mut self, value: u8) {
        self.pulse_width = (self.pulse_width & 0x00FF) | (((value & 0x0F) as u16) << 8);
        self.oscillator.set_pulse_width(self.pulse_width);
    }

    /// Set control register (waveform, gate, sync, ring mod, test)
    pub fn set_control(&mut self, value: u8) {
        // Bit 0: Gate
        let new_gate = (value & 0x01) != 0;
        if new_gate != self.gate {
            self.gate = new_gate;
            self.envelope.set_gate(new_gate);
        }

        // Bit 1: Sync
        self.sync = (value & 0x02) != 0;

        // Bit 2: Ring modulation
        self.ring_mod = (value & 0x04) != 0;

        // Bit 3: Test
        let new_test = (value & 0x08) != 0;
        if new_test != self.test {
            self.test = new_test;
            self.oscillator.set_test(new_test);
        }

        // Bits 4-7: Waveform selection
        self.waveform = match value & 0xF0 {
            0x10 => Waveform::Triangle,
            0x20 => Waveform::Sawtooth,
            0x40 => Waveform::Pulse,
            0x80 => Waveform::Noise,
            _ => Waveform::None, // Multiple or no waveforms selected
        };
        self.oscillator.set_waveform(self.waveform);
    }

    /// Set attack and decay values
    pub fn set_attack_decay(&mut self, value: u8) {
        self.envelope.set_attack_decay(value);
    }

    /// Set sustain and release values
    pub fn set_sustain_release(&mut self, value: u8) {
        self.envelope.set_sustain_release(value);
    }

    /// Clock the voice (oscillator and envelope)
    pub fn clock(&mut self, sync_source_zero: bool) {
        // Clock oscillator with sync control
        let sync_reset = self.sync && sync_source_zero;
        self.oscillator.clock(sync_reset);

        // Clock envelope
        self.envelope.clock();
    }

    /// Check if oscillator crossed zero (for syncing other voices)
    pub fn crossed_zero(&self) -> bool {
        self.oscillator.crossed_zero()
    }

    /// Get voice output (oscillator * envelope)
    pub fn output(&self) -> i16 {
        // Get oscillator output (12-bit signed: -2048 to 2047)
        let osc_output = if self.ring_mod {
            // Ring modulation affects oscillator output
            // This is handled in the main SID mixing, passing the source
            self.oscillator.output(None)
        } else {
            self.oscillator.output(None)
        };

        // Get envelope output (8-bit: 0-255)
        let env_output = self.envelope.output() as i32;

        // Multiply oscillator by envelope
        // Oscillator is 12-bit signed, envelope is 8-bit unsigned
        // Result is 20-bit, we scale down to 16-bit signed
        let result = (osc_output as i32 * env_output) / 256;

        result as i16
    }

    /// Get oscillator output for OSC3 register read
    pub fn oscillator_output(&self) -> u8 {
        self.oscillator.output_byte()
    }

    /// Get envelope output for ENV3 register read
    pub fn envelope_output(&self) -> u8 {
        self.envelope.output_byte()
    }

    /// Get voice output with ring modulation source
    pub fn output_with_ring_mod(&self, ring_mod_source: &Voice) -> i16 {
        let osc_output = if self.ring_mod {
            self.oscillator.output(Some(&ring_mod_source.oscillator))
        } else {
            self.oscillator.output(None)
        };

        let env_output = self.envelope.output() as i32;
        let result = (osc_output as i32 * env_output) / 256;
        result as i16
    }

    /// Reset voice to initial state
    pub fn reset(&mut self) {
        self.oscillator.reset();
        self.envelope.reset();
        self.frequency = 0;
        self.pulse_width = 0;
        self.waveform = Waveform::None;
        self.gate = false;
        self.sync = false;
        self.ring_mod = false;
        self.test = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_new() {
        let voice = Voice::new(0);
        assert_eq!(voice.id, 0);
        assert_eq!(voice.frequency, 0);
    }

    #[test]
    fn test_voice_frequency() {
        let mut voice = Voice::new(0);
        voice.set_freq_lo(0x34);
        voice.set_freq_hi(0x12);
        assert_eq!(voice.frequency(), 0x1234);
    }

    #[test]
    fn test_voice_pulse_width() {
        let mut voice = Voice::new(0);
        voice.set_pw_lo(0xFF);
        voice.set_pw_hi(0x0F);
        assert_eq!(voice.pulse_width, 0x0FFF); // 12-bit max
    }

    #[test]
    fn test_voice_control_gate() {
        let mut voice = Voice::new(0);
        voice.set_control(0x01); // Gate on
        assert!(voice.gate);
        
        voice.set_control(0x00); // Gate off
        assert!(!voice.gate);
    }

    #[test]
    fn test_voice_control_waveforms() {
        let mut voice = Voice::new(0);
        
        voice.set_control(0x10); // Triangle
        assert_eq!(voice.waveform, Waveform::Triangle);
        
        voice.set_control(0x20); // Sawtooth
        assert_eq!(voice.waveform, Waveform::Sawtooth);
        
        voice.set_control(0x40); // Pulse
        assert_eq!(voice.waveform, Waveform::Pulse);
        
        voice.set_control(0x80); // Noise
        assert_eq!(voice.waveform, Waveform::Noise);
    }

    #[test]
    fn test_voice_control_sync() {
        let mut voice = Voice::new(0);
        voice.set_control(0x02); // Sync on
        assert!(voice.sync);
    }

    #[test]
    fn test_voice_control_ring_mod() {
        let mut voice = Voice::new(0);
        voice.set_control(0x04); // Ring mod on
        assert!(voice.ring_mod);
    }

    #[test]
    fn test_voice_control_test() {
        let mut voice = Voice::new(0);
        voice.set_control(0x08); // Test bit on
        assert!(voice.test);
    }

    #[test]
    fn test_voice_output_silent() {
        let voice = Voice::new(0);
        // No gate, envelope should be 0
        assert_eq!(voice.output(), 0);
    }

    #[test]
    fn test_voice_clock() {
        let mut voice1 = Voice::new(0);
        
        voice1.set_freq_lo(0x00);
        voice1.set_freq_hi(0x10);
        
        // Just verify clocking doesn't crash
        voice1.clock(false);
    }
}
