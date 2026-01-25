/// CIA Base Structure
///
/// Common functionality shared between CIA1 and CIA2
use super::timer::Timer;

// Register offsets
pub const REG_PRA: u8 = 0x00;
pub const REG_PRB: u8 = 0x01;
pub const REG_DDRA: u8 = 0x02;
pub const REG_DDRB: u8 = 0x03;
pub const REG_TALO: u8 = 0x04;
pub const REG_TAHI: u8 = 0x05;
pub const REG_TBLO: u8 = 0x06;
pub const REG_TBHI: u8 = 0x07;
pub const REG_TOD_10THS: u8 = 0x08;
pub const REG_TOD_SEC: u8 = 0x09;
pub const REG_TOD_MIN: u8 = 0x0A;
pub const REG_TOD_HR: u8 = 0x0B;
pub const REG_SDR: u8 = 0x0C;
pub const REG_ICR: u8 = 0x0D;
pub const REG_CRA: u8 = 0x0E;
pub const REG_CRB: u8 = 0x0F;

// ICR bits
pub const ICR_TIMER_A: u8 = 0x01;
pub const ICR_TIMER_B: u8 = 0x02;
pub const ICR_TOD_ALARM: u8 = 0x04;
pub const ICR_SDR: u8 = 0x08;
pub const ICR_FLAG: u8 = 0x10;
pub const ICR_ANY: u8 = 0x80;

pub struct CiaBase {
    /// Port A data
    pub pra: u8,
    /// Port B data
    pub prb: u8,
    /// Port A direction (1=output, 0=input)
    pub ddra: u8,
    /// Port B direction (1=output, 0=input)
    pub ddrb: u8,

    /// Timer A
    pub timer_a: Timer,
    /// Timer B
    pub timer_b: Timer,

    /// Interrupt data register (latched interrupts)
    pub icr_data: u8,
    /// Interrupt mask register (enabled interrupts)
    pub icr_mask: u8,

    /// Cycle tracking for synchronization
    pub last_cycles: u64,
}

impl CiaBase {
    pub fn new() -> Self {
        Self {
            pra: 0xFF,
            prb: 0xFF,
            ddra: 0x00,
            ddrb: 0x00,
            timer_a: Timer::new(),
            timer_b: Timer::new(),
            icr_data: 0,
            icr_mask: 0,
            last_cycles: 0,
        }
    }

    /// Read a register
    /// Note: Override this in CIA1/CIA2 for port-specific behavior
    pub fn read(&mut self, reg: u8) -> u8 {
        match reg & 0x0F {
            REG_PRA => {
                // Inputs (0 in DDR) read external value, outputs read internal value
                // For base implementation, return internal value OR'd with input pins
                (self.pra & self.ddra) | (!self.ddra & 0xFF)
            }
            REG_PRB => {
                // Same logic as PRA
                (self.prb & self.ddrb) | (!self.ddrb & 0xFF)
            }
            REG_DDRA => self.ddra,
            REG_DDRB => self.ddrb,
            REG_TALO => self.timer_a.counter_lo(),
            REG_TAHI => self.timer_a.counter_hi(),
            REG_TBLO => self.timer_b.counter_lo(),
            REG_TBHI => self.timer_b.counter_hi(),
            REG_ICR => {
                // Reading ICR clears the data register
                let value = self.icr_data;
                self.icr_data = 0;
                value
            }
            REG_CRA => self.timer_a.control_register(),
            REG_CRB => self.timer_b.control_register(),
            _ => 0xFF, // TOD and SDR not implemented
        }
    }

    /// Write a register
    pub fn write(&mut self, reg: u8, value: u8) {
        match reg & 0x0F {
            REG_PRA => self.pra = value,
            REG_PRB => self.prb = value,
            REG_DDRA => self.ddra = value,
            REG_DDRB => self.ddrb = value,
            REG_TALO => self.timer_a.set_latch_lo(value),
            REG_TAHI => self.timer_a.set_latch_hi(value),
            REG_TBLO => self.timer_b.set_latch_lo(value),
            REG_TBHI => self.timer_b.set_latch_hi(value),
            REG_ICR => {
                // Bit 7 set = enable interrupts, clear = disable
                if value & 0x80 != 0 {
                    self.icr_mask |= value & 0x1F; // Set mask bits
                } else {
                    self.icr_mask &= !(value & 0x1F); // Clear mask bits
                }
            }
            REG_CRA => self.timer_a.set_control(value),
            REG_CRB => self.timer_b.set_control(value),
            _ => {} // TOD and SDR not implemented
        }
    }

    /// Update timers with cycle delta
    pub fn update_timers(&mut self, cycles: u64) -> (bool, bool) {
        let timer_a_underflow = self.timer_a.tick(cycles);
        let timer_b_underflow = self.timer_b.tick(cycles);

        // Latch interrupts in ICR data register
        if timer_a_underflow {
            self.icr_data |= ICR_TIMER_A;
        }
        if timer_b_underflow {
            self.icr_data |= ICR_TIMER_B;
        }

        (timer_a_underflow, timer_b_underflow)
    }

    /// Check if an interrupt should be triggered
    /// Returns true if any enabled interrupt has occurred
    pub fn check_interrupt(&mut self) -> bool {
        let interrupt_occurred = (self.icr_data & self.icr_mask & 0x1F) != 0;
        if interrupt_occurred {
            self.icr_data |= ICR_ANY; // Set bit 7
        }
        interrupt_occurred
    }

    /// Reset the CIA to power-on state
    pub fn reset(&mut self) {
        self.pra = 0xFF;
        self.prb = 0xFF;
        self.ddra = 0x00;
        self.ddrb = 0x00;
        self.timer_a = Timer::new();
        self.timer_b = Timer::new();
        self.icr_data = 0;
        self.icr_mask = 0;
        self.last_cycles = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cia_base_new() {
        let cia = CiaBase::new();
        assert_eq!(cia.pra, 0xFF);
        assert_eq!(cia.prb, 0xFF);
        assert_eq!(cia.ddra, 0x00);
        assert_eq!(cia.ddrb, 0x00);
    }

    #[test]
    fn test_port_direction() {
        let mut cia = CiaBase::new();

        // Set DDRA to output
        cia.write(REG_DDRA, 0xFF);
        assert_eq!(cia.ddra, 0xFF);

        // Write to PRA
        cia.write(REG_PRA, 0x42);
        assert_eq!(cia.read(REG_PRA), 0x42);
    }

    #[test]
    fn test_icr_read_clears() {
        let mut cia = CiaBase::new();
        cia.icr_data = 0x81; // Set some interrupt flags

        let value = cia.read(REG_ICR);
        assert_eq!(value, 0x81);

        let value2 = cia.read(REG_ICR);
        assert_eq!(value2, 0x00); // Should be cleared
    }

    #[test]
    fn test_icr_mask_set() {
        let mut cia = CiaBase::new();

        // Enable timer A interrupt
        cia.write(REG_ICR, 0x81); // Bit 7 set = enable, bit 0 = timer A
        assert_eq!(cia.icr_mask & ICR_TIMER_A, ICR_TIMER_A);
    }

    #[test]
    fn test_icr_mask_clear() {
        let mut cia = CiaBase::new();

        // Enable timer A interrupt
        cia.write(REG_ICR, 0x81);

        // Disable timer A interrupt
        cia.write(REG_ICR, 0x01); // Bit 7 clear = disable
        assert_eq!(cia.icr_mask & ICR_TIMER_A, 0);
    }

    #[test]
    fn test_timer_interrupt() {
        let mut cia = CiaBase::new();

        // Enable timer A interrupt
        cia.write(REG_ICR, 0x81);

        // Set up timer A to underflow quickly
        cia.write(REG_TALO, 0x02);
        cia.write(REG_TAHI, 0x00);
        cia.write(REG_CRA, 0x11); // Start timer + force load

        // Run for 3 cycles (should cause underflow)
        cia.update_timers(3);

        // Check interrupt triggered
        assert!(cia.check_interrupt());
        assert_eq!(cia.icr_data & ICR_ANY, ICR_ANY);
        assert_eq!(cia.icr_data & ICR_TIMER_A, ICR_TIMER_A);
    }

    #[test]
    fn test_reset() {
        let mut cia = CiaBase::new();

        // Modify some state
        cia.write(REG_PRA, 0x42);
        cia.write(REG_DDRA, 0xFF);
        cia.icr_data = 0x81;

        // Reset
        cia.reset();

        // Check defaults restored
        assert_eq!(cia.pra, 0xFF);
        assert_eq!(cia.ddra, 0x00);
        assert_eq!(cia.icr_data, 0x00);
    }
}
