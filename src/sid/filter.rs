/// Filter modes for the SID filter
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FilterMode {
    Off,
    LowPass,
    BandPass,
    HighPass,
    LowPassBandPass,      // LP + BP
    LowPassHighPass,      // LP + HP (notch)
    BandPassHighPass,     // BP + HP
    All,                  // LP + BP + HP
}

impl FilterMode {
    /// Create filter mode from register bits
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x07 {
            0x00 => FilterMode::Off,
            0x01 => FilterMode::LowPass,
            0x02 => FilterMode::BandPass,
            0x03 => FilterMode::LowPassBandPass,
            0x04 => FilterMode::HighPass,
            0x05 => FilterMode::LowPassHighPass,
            0x06 => FilterMode::BandPassHighPass,
            0x07 => FilterMode::All,
            _ => FilterMode::Off,
        }
    }
}

/// SID filter implementation using state-variable filter
pub struct Filter {
    // Filter parameters
    cutoff: f32,              // Normalized cutoff frequency (0.0-1.0)
    resonance: f32,           // Resonance/Q factor (0.0-1.0)
    mode: FilterMode,

    // State-variable filter state
    low_pass: f32,
    band_pass: f32,
    high_pass: f32,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            cutoff: 0.0,
            resonance: 0.0,
            mode: FilterMode::Off,
            low_pass: 0.0,
            band_pass: 0.0,
            high_pass: 0.0,
        }
    }

    /// Set cutoff frequency from 11-bit register value
    pub fn set_cutoff(&mut self, value: u16) {
        // SID cutoff is highly non-linear
        // This is a simplified approximation
        let normalized = (value & 0x7FF) as f32 / 2048.0;
        
        // Apply power curve to approximate SID's non-linear response
        self.cutoff = normalized.powf(2.0) * 0.45; // Max around 0.45 to avoid aliasing
    }

    /// Set resonance from 4-bit register value
    pub fn set_resonance(&mut self, value: u8) {
        // Resonance: 0-15 maps to damping factor
        // Higher resonance = lower damping
        let res_normalized = (value & 0x0F) as f32 / 15.0;
        
        // Map to Q factor: low resonance = high damping (low Q)
        self.resonance = 1.0 - res_normalized * 0.95;
    }

    /// Set filter mode
    pub fn set_mode(&mut self, mode: FilterMode) {
        self.mode = mode;
    }

    /// Process a sample through the filter
    pub fn process(&mut self, input: f32) -> f32 {
        if self.mode == FilterMode::Off {
            return input;
        }

        // State-variable filter implementation
        // Based on Hal Chamberlin's design
        
        // Update low-pass
        self.low_pass += self.cutoff * self.band_pass;
        
        // Update high-pass
        self.high_pass = input - self.low_pass - self.resonance * self.band_pass;
        
        // Update band-pass
        self.band_pass += self.cutoff * self.high_pass;

        // Select output based on mode
        match self.mode {
            FilterMode::Off => input,
            FilterMode::LowPass => self.low_pass,
            FilterMode::BandPass => self.band_pass,
            FilterMode::HighPass => self.high_pass,
            FilterMode::LowPassBandPass => self.low_pass + self.band_pass,
            FilterMode::LowPassHighPass => self.low_pass + self.high_pass,
            FilterMode::BandPassHighPass => self.band_pass + self.high_pass,
            FilterMode::All => self.low_pass + self.band_pass + self.high_pass,
        }
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.cutoff = 0.0;
        self.resonance = 0.0;
        self.mode = FilterMode::Off;
        self.low_pass = 0.0;
        self.band_pass = 0.0;
        self.high_pass = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_new() {
        let filter = Filter::new();
        assert_eq!(filter.mode, FilterMode::Off);
        assert_eq!(filter.cutoff, 0.0);
    }

    #[test]
    fn test_filter_mode_from_bits() {
        assert_eq!(FilterMode::from_bits(0x00), FilterMode::Off);
        assert_eq!(FilterMode::from_bits(0x01), FilterMode::LowPass);
        assert_eq!(FilterMode::from_bits(0x02), FilterMode::BandPass);
        assert_eq!(FilterMode::from_bits(0x04), FilterMode::HighPass);
        assert_eq!(FilterMode::from_bits(0x07), FilterMode::All);
    }

    #[test]
    fn test_filter_set_cutoff() {
        let mut filter = Filter::new();
        filter.set_cutoff(0x400); // Mid-point
        assert!(filter.cutoff > 0.0);
        assert!(filter.cutoff < 0.5);
    }

    #[test]
    fn test_filter_set_resonance() {
        let mut filter = Filter::new();
        filter.set_resonance(15); // Max resonance
        assert!(filter.resonance < 0.1); // Should be low damping
        
        filter.set_resonance(0); // Min resonance
        assert!(filter.resonance > 0.9); // Should be high damping
    }

    #[test]
    fn test_filter_process_off() {
        let mut filter = Filter::new();
        filter.set_mode(FilterMode::Off);
        let input = 1000.0;
        assert_eq!(filter.process(input), input);
    }

    #[test]
    fn test_filter_process_lowpass() {
        let mut filter = Filter::new();
        filter.set_mode(FilterMode::LowPass);
        filter.set_cutoff(0x200);
        filter.set_resonance(4);
        
        // Process some samples
        for _ in 0..10 {
            filter.process(1000.0);
        }
        
        // Low-pass should have some output
        assert!(filter.low_pass != 0.0);
    }

    #[test]
    fn test_filter_process_highpass() {
        let mut filter = Filter::new();
        filter.set_mode(FilterMode::HighPass);
        filter.set_cutoff(0x200);
        
        // High-pass should respond to input
        let _output = filter.process(1000.0);
        assert!(filter.high_pass != 0.0);
    }

    #[test]
    fn test_filter_reset() {
        let mut filter = Filter::new();
        filter.set_cutoff(0x400);
        filter.set_resonance(8);
        filter.set_mode(FilterMode::LowPass);
        filter.process(1000.0);
        
        filter.reset();
        
        assert_eq!(filter.cutoff, 0.0);
        assert_eq!(filter.resonance, 0.0);
        assert_eq!(filter.mode, FilterMode::Off);
        assert_eq!(filter.low_pass, 0.0);
    }

    #[test]
    fn test_filter_combined_modes() {
        let mut filter = Filter::new();
        filter.set_mode(FilterMode::LowPassBandPass);
        filter.set_cutoff(0x300);
        
        let _output = filter.process(500.0);
        // Just verify it doesn't crash with combined modes
    }
}
