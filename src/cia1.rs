use super::cia::{CiaBase, REG_PRB};
use super::cpu::Cpu;
use super::io::IO;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Cia1 {
    base: CiaBase,
    cpu: Weak<RefCell<Cpu>>,
    io: Weak<RefCell<IO>>,
}

impl Cia1 {
    pub fn new(cpu: Weak<RefCell<Cpu>>, io: Weak<RefCell<IO>>) -> Self {
        let mut base = CiaBase::new();
        // CIA1: PRA is output for keyboard column selection, PRB is input for row reading
        base.ddra = 0xFF; // All output
        base.ddrb = 0x00; // All input
        Cia1 {
            base,
            cpu,
            io,
        }
    }

    pub fn write_register(&mut self, r: u8, v: u8) {
        self.base.write(r, v);
    }

    pub fn read_register(&mut self, r: u8) -> u8 {
        match r & 0x0F {
            REG_PRB => {
                // Keyboard matrix read
                // Scan keyboard based on column selection in PRA
                if let Some(io) = self.io.upgrade() {
                    let col_mask = !self.base.pra; // Active low - bits set = columns selected
                    if col_mask == 0 {
                        // No columns selected, return all high
                        return 0xFF;
                    }
                    
                    // Read all selected columns and OR them together
                    let mut result = 0xFF;
                    for col in 0..8 {
                        if (col_mask & (1 << col)) != 0 {
                            // This column is selected, AND its rows with result
                            result &= io.borrow().keyboard_matrix_row(col);
                        }
                    }
                    return result;
                }
                0xFF
            }
            _ => self.base.read(r),
        }
    }

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
            // Trigger IRQ on CPU
            if let Some(cpu) = self.cpu.upgrade() {
                cpu.borrow_mut().irq();
            }
        }

        true
    }

    pub fn reset(&mut self) {
        self.base.reset();
    }
}

