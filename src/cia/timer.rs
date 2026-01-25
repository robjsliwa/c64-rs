/// CIA Timer Implementation
///
/// Each CIA chip has two 16-bit timers (Timer A and Timer B).
/// Timers count down from a latch value and can generate interrupts on underflow.

pub struct Timer {
    /// Current counter value (counts down)
    counter: u16,
    /// Latch value (reload value)
    latch: u16,
    /// Timer is running
    running: bool,
    /// One-shot mode (vs continuous)
    one_shot: bool,
    /// Output state (for timer B CNT mode and PB6/7 output)
    output: bool,
    /// Underflow occurred this cycle
    underflow: bool,
    /// Output timer to PB6/7
    output_to_pb: bool,
    /// Toggle mode (vs pulse mode) for PB6/7
    toggle_mode: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            counter: 0xFFFF,
            latch: 0xFFFF,
            running: false,
            one_shot: false,
            output: false,
            underflow: false,
            output_to_pb: false,
            toggle_mode: false,
        }
    }

    /// Tick the timer for the given number of cycles
    /// Returns true if underflow occurred
    pub fn tick(&mut self, cycles: u64) -> bool {
        if !self.running {
            return false;
        }

        self.underflow = false;

        for _ in 0..cycles {
            // Decrement counter
            if self.counter > 0 {
                self.counter = self.counter.wrapping_sub(1);
            } else {
                // Counter is at 0, this is underflow
                self.underflow = true;
                
                // Toggle or pulse the output
                if self.toggle_mode {
                    self.output = !self.output;
                } else {
                    self.output = true; // Pulse high for one cycle
                }

                // Reload from latch
                self.counter = self.latch;

                // Stop if one-shot mode
                if self.one_shot {
                    self.running = false;
                    break; // Exit the loop after stopping
                }
            }

            // Reset pulse output after one cycle
            if !self.toggle_mode && self.output && !self.underflow {
                self.output = false;
            }
        }

        self.underflow
    }

    /// Set the low byte of the latch
    pub fn set_latch_lo(&mut self, value: u8) {
        self.latch = (self.latch & 0xFF00) | value as u16;
    }

    /// Set the high byte of the latch
    pub fn set_latch_hi(&mut self, value: u8) {
        self.latch = (self.latch & 0x00FF) | ((value as u16) << 8);
    }

    /// Get the low byte of the counter
    pub fn counter_lo(&self) -> u8 {
        (self.counter & 0xFF) as u8
    }

    /// Get the high byte of the counter
    pub fn counter_hi(&self) -> u8 {
        (self.counter >> 8) as u8
    }

    /// Set the control register (CRA or CRB)
    ///
    /// Bit 0: Start timer (1=start, 0=stop)
    /// Bit 1: Output to PB6/7 (1=yes, 0=no)
    /// Bit 2: Toggle vs Pulse (1=toggle, 0=pulse)
    /// Bit 3: One-shot vs Continuous (1=one-shot, 0=continuous)
    /// Bit 4: Force load (1=load latch into counter)
    /// Bits 5-6: Timer B input mode (for Timer B only)
    pub fn set_control(&mut self, value: u8) {
        self.running = value & 0x01 != 0;
        self.output_to_pb = value & 0x02 != 0;
        self.toggle_mode = value & 0x04 != 0;
        self.one_shot = value & 0x08 != 0;

        // Force load (bit 4)
        if value & 0x10 != 0 {
            self.counter = self.latch;
        }
    }

    /// Get the control register value
    pub fn control_register(&self) -> u8 {
        let mut value = 0u8;
        if self.running {
            value |= 0x01;
        }
        if self.output_to_pb {
            value |= 0x02;
        }
        if self.toggle_mode {
            value |= 0x04;
        }
        if self.one_shot {
            value |= 0x08;
        }
        value
    }

    /// Check if underflow occurred
    pub fn underflow(&self) -> bool {
        self.underflow
    }

    /// Get the output state (for PB6/7 or Timer B CNT input)
    pub fn output(&self) -> bool {
        self.output
    }

    /// Check if timer is running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_new() {
        let timer = Timer::new();
        assert_eq!(timer.counter, 0xFFFF);
        assert_eq!(timer.latch, 0xFFFF);
        assert!(!timer.running);
    }

    #[test]
    fn test_timer_latch() {
        let mut timer = Timer::new();
        timer.set_latch_lo(0x34);
        timer.set_latch_hi(0x12);
        assert_eq!(timer.latch, 0x1234);
    }

    #[test]
    fn test_timer_countdown() {
        let mut timer = Timer::new();
        timer.set_latch_lo(0x05);
        timer.set_latch_hi(0x00);
        timer.set_control(0x11); // Start timer + force load
        
        assert!(!timer.tick(5)); // Count down 5 cycles, no underflow yet
        assert_eq!(timer.counter, 0x00);
        
        assert!(timer.tick(1)); // Underflow on this cycle
        assert_eq!(timer.counter, 0x05); // Reloaded from latch
    }

    #[test]
    fn test_timer_one_shot() {
        let mut timer = Timer::new();
        timer.set_latch_lo(0x02);
        timer.set_latch_hi(0x00);
        timer.set_control(0x19); // Start timer + one-shot mode + force load
        
        timer.tick(3); // Count down to underflow
        assert!(!timer.is_running()); // Should have stopped
    }

    #[test]
    fn test_timer_continuous() {
        let mut timer = Timer::new();
        timer.set_latch_lo(0x02);
        timer.set_latch_hi(0x00);
        timer.set_control(0x11); // Start timer (continuous mode) + force load
        
        timer.tick(3); // Count down to underflow
        assert!(timer.is_running()); // Should still be running
        assert_eq!(timer.counter, 0x02); // Reloaded
    }

    #[test]
    fn test_force_load() {
        let mut timer = Timer::new();
        timer.set_latch_lo(0x42);
        timer.set_latch_hi(0x13);
        
        timer.set_control(0x10); // Force load
        assert_eq!(timer.counter, 0x1342);
    }

    #[test]
    fn test_control_register() {
        let mut timer = Timer::new();
        timer.set_control(0x0F); // Set all control bits
        
        let cr = timer.control_register();
        assert_eq!(cr & 0x0F, 0x0F);
    }
}
