/// ADSR envelope states
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnvelopeState {
    Attack,
    Decay,
    Sustain,
    Release,
}

/// ADSR envelope generator
pub struct Envelope {
    state: EnvelopeState,
    level: u8,                // 8-bit output level (0-255)
    
    // ADSR parameters (all 4-bit: 0-15)
    attack: u8,
    decay: u8,
    sustain: u8,              // Sustain level, not rate
    release: u8,
    
    // Rate counters
    rate_counter: u16,
    exponential_counter: u8,  // For exponential decay/release
    
    // Gate control
    gate: bool,
}

// SID ADSR rate table (cycles per step)
// These values determine how fast envelope stages progress
const RATE_TABLE: [u16; 16] = [
    9, 32, 63, 95, 149, 220, 267, 313,
    392, 977, 1954, 3126, 3907, 11720, 19532, 31251,
];

// Exponential counter comparison values
const EXPONENTIAL_TABLE: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        // Exponential decay: counter compared against envelope level
        table[i] = match i {
            0..=93 => 1,
            94..=149 => 2,
            150..=221 => 4,
            222..=253 => 8,
            _ => 16,
        };
        i += 1;
    }
    table
};

impl Envelope {
    pub fn new() -> Self {
        Self {
            state: EnvelopeState::Release,
            level: 0,
            attack: 0,
            decay: 0,
            sustain: 0,
            release: 0,
            rate_counter: 0,
            exponential_counter: 0,
            gate: false,
        }
    }

    /// Set attack and decay from register value
    pub fn set_attack_decay(&mut self, value: u8) {
        self.attack = value >> 4;
        self.decay = value & 0x0F;
    }

    /// Set sustain and release from register value
    pub fn set_sustain_release(&mut self, value: u8) {
        self.sustain = value >> 4;
        self.release = value & 0x0F;
    }

    /// Gate on - start attack phase
    pub fn gate_on(&mut self) {
        if !self.gate {
            self.gate = true;
            self.state = EnvelopeState::Attack;
            self.rate_counter = 0;
        }
    }

    /// Gate off - start release phase
    pub fn gate_off(&mut self) {
        if self.gate {
            self.gate = false;
            self.state = EnvelopeState::Release;
            self.rate_counter = 0;
        }
    }

    /// Set gate state directly
    pub fn set_gate(&mut self, gate: bool) {
        if gate {
            self.gate_on();
        } else {
            self.gate_off();
        }
    }

    /// Clock the envelope generator
    pub fn clock(&mut self) {
        self.rate_counter = self.rate_counter.wrapping_add(1);

        let rate = match self.state {
            EnvelopeState::Attack => RATE_TABLE[self.attack as usize],
            EnvelopeState::Decay => RATE_TABLE[self.decay as usize],
            EnvelopeState::Sustain => return, // No change in sustain
            EnvelopeState::Release => RATE_TABLE[self.release as usize],
        };

        if self.rate_counter >= rate {
            self.rate_counter = 0;

            match self.state {
                EnvelopeState::Attack => {
                    // Linear attack
                    if self.level < 0xFF {
                        self.level = self.level.saturating_add(1);
                    }
                    if self.level == 0xFF {
                        self.state = EnvelopeState::Decay;
                        self.exponential_counter = 0;
                    }
                }

                EnvelopeState::Decay => {
                    // Exponential decay to sustain level
                    let sustain_level = (self.sustain << 4) | self.sustain;
                    
                    if self.level > sustain_level {
                        self.exponential_counter = self.exponential_counter.wrapping_add(1);
                        
                        if self.exponential_counter >= EXPONENTIAL_TABLE[self.level as usize] {
                            self.exponential_counter = 0;
                            self.level = self.level.saturating_sub(1);
                        }
                    } else {
                        self.state = EnvelopeState::Sustain;
                    }
                }

                EnvelopeState::Release => {
                    // Exponential release to zero
                    if self.level > 0 {
                        self.exponential_counter = self.exponential_counter.wrapping_add(1);
                        
                        if self.exponential_counter >= EXPONENTIAL_TABLE[self.level as usize] {
                            self.exponential_counter = 0;
                            self.level = self.level.saturating_sub(1);
                        }
                    }
                }

                EnvelopeState::Sustain => {
                    // Hold at sustain level
                }
            }
        }
    }

    /// Get envelope output (8-bit)
    pub fn output(&self) -> u8 {
        self.level
    }

    /// Get output as byte for ENV3 register read
    pub fn output_byte(&self) -> u8 {
        self.level
    }

    /// Get current envelope state
    pub fn state(&self) -> EnvelopeState {
        self.state
    }

    /// Reset envelope
    pub fn reset(&mut self) {
        self.state = EnvelopeState::Release;
        self.level = 0;
        self.rate_counter = 0;
        self.exponential_counter = 0;
        self.gate = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_new() {
        let env = Envelope::new();
        assert_eq!(env.level, 0);
        assert_eq!(env.state(), EnvelopeState::Release);
    }

    #[test]
    fn test_envelope_attack_decay() {
        let mut env = Envelope::new();
        env.set_attack_decay(0x9A); // Attack=9, Decay=10
        assert_eq!(env.attack, 9);
        assert_eq!(env.decay, 10);
    }

    #[test]
    fn test_envelope_sustain_release() {
        let mut env = Envelope::new();
        env.set_sustain_release(0x7B); // Sustain=7, Release=11
        assert_eq!(env.sustain, 7);
        assert_eq!(env.release, 11);
    }

    #[test]
    fn test_envelope_gate_on() {
        let mut env = Envelope::new();
        env.gate_on();
        assert_eq!(env.state(), EnvelopeState::Attack);
        assert!(env.gate);
    }

    #[test]
    fn test_envelope_gate_off() {
        let mut env = Envelope::new();
        env.gate_on();
        env.gate_off();
        assert_eq!(env.state(), EnvelopeState::Release);
        assert!(!env.gate);
    }

    #[test]
    fn test_envelope_attack_phase() {
        let mut env = Envelope::new();
        env.set_attack_decay(0x00); // Fastest attack
        env.gate_on();
        
        // Clock through attack phase
        for _ in 0..10 {
            env.clock();
        }
        
        // Level should increase
        assert!(env.level > 0);
    }

    #[test]
    fn test_envelope_full_cycle() {
        let mut env = Envelope::new();
        env.set_attack_decay(0x00); // Fast attack/decay
        env.set_sustain_release(0x80); // Sustain=8, fast release
        
        // Attack
        env.gate_on();
        for _ in 0..(9 * 256) { // Should reach max
            env.clock();
        }
        assert_eq!(env.level, 0xFF);
        
        // Should transition to decay
        assert_eq!(env.state(), EnvelopeState::Decay);
    }

    #[test]
    fn test_envelope_output() {
        let mut env = Envelope::new();
        env.level = 128;
        assert_eq!(env.output(), 128);
    }

    #[test]
    fn test_rate_table_values() {
        // Verify rate table has correct number of entries
        assert_eq!(RATE_TABLE.len(), 16);
        // Verify it's monotonically increasing
        for i in 1..16 {
            assert!(RATE_TABLE[i] > RATE_TABLE[i-1]);
        }
    }
}
