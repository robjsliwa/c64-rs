/// Waveform types for the SID oscillator
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Waveform {
    None,
    Triangle,
    Sawtooth,
    Pulse,
    Noise,
}

/// Oscillator generates audio waveforms
pub struct Oscillator {
    phase: u32,     // 24-bit phase accumulator
    frequency: u16, // 16-bit frequency control
    waveform: Waveform,
    pulse_width: u16, // 12-bit pulse width (0-4095)
    test_bit: bool,   // Test bit stops oscillator

    // For noise waveform
    noise_lfsr: u32,  // 23-bit LFSR
    noise_output: u8, // Cached noise output

    // For sync
    prev_phase_msb: bool, // Previous MSB for zero-crossing detection
}

impl Oscillator {
    pub fn new() -> Self {
        Self {
            phase: 0,
            frequency: 0,
            waveform: Waveform::None,
            pulse_width: 0,
            test_bit: false,
            noise_lfsr: 0x7FFFF8, // Initial LFSR value
            noise_output: 0,
            prev_phase_msb: false,
        }
    }

    /// Set the frequency control value (16-bit)
    pub fn set_frequency(&mut self, freq: u16) {
        self.frequency = freq;
    }

    /// Get current frequency
    pub fn frequency(&self) -> u16 {
        self.frequency
    }

    /// Set waveform type
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Set pulse width (12-bit value)
    pub fn set_pulse_width(&mut self, pw: u16) {
        self.pulse_width = pw & 0x0FFF;
    }

    /// Set test bit (stops oscillator when set)
    pub fn set_test(&mut self, test: bool) {
        if test {
            self.phase = 0;
            self.noise_lfsr = 0x7FFFF8;
        }
        self.test_bit = test;
    }

    /// Clock the oscillator by one cycle
    pub fn clock(&mut self, sync_reset: bool) {
        if self.test_bit {
            return; // Test bit stops oscillator
        }

        // Store previous MSB for zero-crossing detection
        self.prev_phase_msb = (self.phase & 0x800000) != 0;

        // Hard sync resets phase
        if sync_reset {
            self.phase = 0;
        } else {
            // Increment phase accumulator
            self.phase = self.phase.wrapping_add(self.frequency as u32);
        }

        // Clock noise LFSR when bit 19 of phase goes high
        if self.waveform == Waveform::Noise {
            if (self.phase & 0x080000) != 0 {
                self.clock_noise_lfsr();
            }
        }
    }

    /// Check if oscillator crossed zero (for sync)
    pub fn crossed_zero(&self) -> bool {
        self.prev_phase_msb && (self.phase & 0x800000) == 0
    }

    /// Get oscillator output value (12-bit signed, -2048 to 2047)
    pub fn output(&self, ring_mod_source: Option<&Oscillator>) -> i16 {
        match self.waveform {
            Waveform::None => 0,
            Waveform::Triangle => self.triangle_output(ring_mod_source),
            Waveform::Sawtooth => self.sawtooth_output(),
            Waveform::Pulse => self.pulse_output(),
            Waveform::Noise => self.noise_output(),
        }
    }

    /// Get output as unsigned byte (for OSC3 register read)
    pub fn output_byte(&self) -> u8 {
        ((self.output(None) + 2048) >> 4) as u8
    }

    /// Triangle wave output
    fn triangle_output(&self, ring_mod_source: Option<&Oscillator>) -> i16 {
        // Use upper 12 bits of 24-bit accumulator
        let phase12 = (self.phase >> 12) as u16;

        // Apply ring modulation if source provided
        let mut output = if let Some(source) = ring_mod_source {
            if (source.phase & 0x800000) != 0 {
                phase12 ^ 0xFFF
            } else {
                phase12
            }
        } else {
            phase12
        };

        // Triangle wave: use MSB (bit 11 of the 12-bit value) to determine direction
        if (output & 0x800) != 0 {
            // Top half - invert to create triangle shape
            output = 0xFFF - (output & 0x7FF);
        }

        // Convert to signed 16-bit centered at 0
        (output as i16) - 2048
    }

    /// Sawtooth wave output
    fn sawtooth_output(&self) -> i16 {
        let phase12 = (self.phase >> 12) as u16;
        (phase12 as i16) - 2048
    }

    /// Pulse wave output
    fn pulse_output(&self) -> i16 {
        let phase12 = (self.phase >> 12) as u16;
        if phase12 < self.pulse_width {
            2047 // High
        } else {
            -2048 // Low
        }
    }

    /// Noise wave output
    fn noise_output(&self) -> i16 {
        (self.noise_output as i16) - 128
    }

    /// Clock the noise LFSR
    fn clock_noise_lfsr(&mut self) {
        // 23-bit LFSR with taps at bits 22 and 17
        let bit22 = (self.noise_lfsr >> 22) & 1;
        let bit17 = (self.noise_lfsr >> 17) & 1;
        let feedback = bit22 ^ bit17;

        self.noise_lfsr = ((self.noise_lfsr << 1) | feedback) & 0x7FFFFF;

        // Update noise output from LFSR bits
        self.noise_output = ((self.noise_lfsr >> 15) & 0xFF) as u8;
    }

    /// Reset oscillator
    pub fn reset(&mut self) {
        self.phase = 0;
        self.frequency = 0;
        self.waveform = Waveform::None;
        self.pulse_width = 0;
        self.test_bit = false;
        self.noise_lfsr = 0x7FFFF8;
        self.noise_output = 0;
        self.prev_phase_msb = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_new() {
        let osc = Oscillator::new();
        assert_eq!(osc.phase, 0);
        assert_eq!(osc.frequency, 0);
    }

    #[test]
    fn test_oscillator_frequency() {
        let mut osc = Oscillator::new();
        osc.set_frequency(440);
        assert_eq!(osc.frequency(), 440);
    }

    #[test]
    fn test_oscillator_clock() {
        let mut osc = Oscillator::new();
        osc.set_frequency(1000);
        let initial_phase = osc.phase;
        osc.clock(false);
        assert_eq!(osc.phase, initial_phase + 1000);
    }

    #[test]
    fn test_oscillator_sync_reset() {
        let mut osc = Oscillator::new();
        osc.set_frequency(1000);
        osc.clock(false);
        assert!(osc.phase > 0);
        osc.clock(true); // Sync reset
        assert_eq!(osc.phase, 0);
    }

    #[test]
    fn test_oscillator_test_bit() {
        let mut osc = Oscillator::new();
        osc.set_frequency(1000);
        osc.set_test(true);
        assert_eq!(osc.phase, 0);
        osc.clock(false);
        assert_eq!(osc.phase, 0); // Should not advance
    }

    #[test]
    fn test_sawtooth_output() {
        let mut osc = Oscillator::new();
        osc.set_waveform(Waveform::Sawtooth);
        osc.phase = 0x800000; // Mid-point
        let output = osc.output(None);
        assert_eq!(output, 0); // Should be centered at 0
    }

    #[test]
    fn test_pulse_output() {
        let mut osc = Oscillator::new();
        osc.set_waveform(Waveform::Pulse);
        osc.set_pulse_width(0x800); // 50% duty cycle

        osc.phase = 0x000000; // Start of cycle
        assert_eq!(osc.output(None), 2047); // High

        osc.phase = 0x900000; // Past midpoint
        assert_eq!(osc.output(None), -2048); // Low
    }

    #[test]
    fn test_triangle_output() {
        let mut osc = Oscillator::new();
        osc.set_waveform(Waveform::Triangle);

        osc.phase = 0x000000; // Bottom of triangle
        let output1 = osc.output(None);
        assert_eq!(output1, -2048);

        osc.phase = 0x800000; // Peak of triangle
        let output2 = osc.output(None);
        assert_eq!(output2, 2047);
    }

    #[test]
    fn test_zero_crossing() {
        let mut osc = Oscillator::new();
        osc.set_frequency(0x8000); // Large frequency value

        assert!(!osc.crossed_zero());

        // Set phase near the MSB transition
        osc.phase = 0x7F0000;
        osc.clock(false);

        // After enough clocks, should eventually cross
        for _ in 0..100 {
            osc.clock(false);
            if osc.crossed_zero() {
                break;
            }
        }
    }
}
