use super::cia::CiaBase;
use super::cpu::Cpu;
use std::cell::RefCell;
use std::rc::Weak;

pub struct Cia2 {
    base: CiaBase,
    cpu: Weak<RefCell<Cpu>>,
}

impl Cia2 {
    pub fn new(cpu: Weak<RefCell<Cpu>>) -> Self {
        Cia2 {
            base: CiaBase::new(),
            cpu,
        }
    }

    pub fn write_register(&mut self, r: u8, v: u8) {
        self.base.write(r, v);
    }

    pub fn read_register(&mut self, r: u8) -> u8 {
        self.base.read(r)
    }

    /// Get VIC bank selection (0-3)
    /// PRA bits 0-1 (inverted) select the VIC memory bank
    #[allow(dead_code)] // TODO: VIC should call this for proper memory banking
    pub fn vic_base_address(&self) -> u16 {
        ((!self.base.pra & 0x3) as u16) << 14
    }

    #[allow(dead_code)] // TODO: Wire up to emulator reset
    pub fn step(&mut self) -> bool {
        // Get current CPU cycles
        let current_cycles = if let Some(cpu) = self.cpu.upgrade() {
            cpu.borrow().cycles() as u64
        } else {
            return true;
        };

        // Calculate cycle delta
        let delta = current_cycles.saturating_sub(self.base.last_cycles);
        self.base.last_cycles = current_cycles;

        // Update timers
        self.base.update_timers(delta);

        // Check for interrupts
        if self.base.check_interrupt() {
            // Trigger NMI on CPU (CIA2 generates NMI, not IRQ)
            if let Some(cpu) = self.cpu.upgrade() {
                cpu.borrow_mut().nmi();
            }
        }

        true
    }

    #[allow(dead_code)] // TODO: Wire up to emulator reset
    pub fn reset(&mut self) {
        self.base.reset();
    }
}
