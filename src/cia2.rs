use super::common::{InputMode, RunMode};
use super::cpu::Cpu;
use std::cell::RefCell;
use std::rc::Rc;

struct Cia2<'a> {
    cpu: Rc<RefCell<Cpu<'a>>>,
    timer_a_latch: u16,
    timer_b_latch: u16,
    timer_a_counter: i16,
    timer_b_counter: i16,
    timer_a_enabled: bool,
    timer_b_enabled: bool,
    timer_a_irq_enabled: bool,
    timer_b_irq_enabled: bool,
    timer_a_irq_triggered: bool,
    timer_b_irq_triggered: bool,
    timer_a_run_mode: u8,
    timer_b_run_mode: u8,
    timer_a_input_mode: u8,
    timer_b_input_mode: u8,
    prev_cpu_cycles: u32,
    pra: u8,
    prb: u8,
}

impl<'a> Cia2<'a> {
    pub fn new(cpu: Rc<RefCell<Cpu<'a>>>) -> Self {
        Cia2 {
            cpu,
            timer_a_latch: 0,
            timer_b_latch: 0,
            timer_a_counter: 0,
            timer_b_counter: 0,
            timer_a_enabled: false,
            timer_b_enabled: false,
            timer_a_irq_enabled: false,
            timer_b_irq_enabled: false,
            timer_a_irq_triggered: false,
            timer_b_irq_triggered: false,
            timer_a_run_mode: RunMode::Restart.as_u8(), // Assuming ModeRestart is the default mode
            timer_b_run_mode: RunMode::Restart.as_u8(), // Assuming ModeRestart is the default mode
            timer_a_input_mode: InputMode::Processor.as_u8(), // Assuming ModeProcessor is the default mode
            timer_b_input_mode: InputMode::Processor.as_u8(), // Assuming ModeProcessor is the default mode
            prev_cpu_cycles: 0,
            pra: 0xff, // Default value as per cia2.cpp
            prb: 0xff, // Default value as per cia2.cpp
        }
    }

    pub fn write_register(&mut self, r: u8, v: u8) {
        match r {
            0x0 => self.pra = v, // Data port A (PRA)
            0x1 => self.prb = v, // Data port B (PRB)
            0x2 => (),           // Data direction port A (DDRA) - Placeholder for implementation
            0x3 => (),           // Data direction port B (DDRB) - Placeholder for implementation
            0x4 => {
                // Timer A low byte
                self.timer_a_latch &= 0xff00;
                self.timer_a_latch |= v as u16;
            }
            0x5 => {
                // Timer A high byte
                self.timer_a_latch &= 0x00ff;
                self.timer_a_latch |= (v as u16) << 8;
            }
            0x6 => {
                // Timer B low byte
                self.timer_b_latch &= 0xff00;
                self.timer_b_latch |= v as u16;
            }
            0x7 => {
                // Timer B high byte
                self.timer_b_latch &= 0x00ff;
                self.timer_b_latch |= (v as u16) << 8;
            }
            0x8 => (),
            0x9 => (),
            0xa => (),
            0xb => (),
            0xc => (),
            0xd => {
                if v & 1 != 0 {
                    self.timer_a_irq_enabled = v & 0x80 != 0;
                }
                if v & 2 != 0 {
                    self.timer_b_irq_enabled = v & 0x80 != 0;
                }
            }
            0xe => {
                self.timer_a_enabled = (v & 1) != 0;
                self.timer_a_input_mode = (v & 0x20) >> 5;
                if (v & 0x10) != 0 {
                    self.timer_a_counter = self.timer_a_latch as i16;
                }
            }
            0xf => {
                self.timer_b_enabled = (v & 1) != 0;
                self.timer_b_input_mode = (v & 0x20) | (v & 0x40) >> 5;
                if (v & 0x10) != 0 {
                    self.timer_b_counter = self.timer_b_latch as i16;
                }
            }
            _ => (),
        }
    }

    pub fn read_register(&self, r: u8) -> u8 {
        let mut retval = 0;
        match r {
            0x0 => self.pra,
            0x1 => self.prb,
            0x2 => 0, // data direction port a (DDRA)
            0x3 => 0, // data direction port b (DDRB)
            0x4 => (self.timer_a_counter & 0x00ff) as u8,
            0x5 => ((self.timer_a_counter as u16 & 0xff00) >> 8) as u8,
            0x6 => (self.timer_b_counter & 0x00ff) as u8,
            0x7 => ((self.timer_b_counter as u16 & 0xff00) >> 8) as u8,
            0x8 => retval,
            0x9 => retval,
            0xa => retval,
            0xb => retval,
            0xc => retval,
            0xd => {
                if self.timer_a_irq_triggered || self.timer_b_irq_triggered {
                    retval |= 1 << 7; // IRQ occurred
                    if self.timer_a_irq_triggered {
                        retval |= 1 << 0;
                    }
                    if self.timer_b_irq_triggered {
                        retval |= 1 << 1;
                    }
                }
                retval
            }
            0xe => retval,
            0xf => retval,
            _ => retval,
        }
    }

    pub fn reset_timer_a(&mut self) {
        match self.timer_a_run_mode {
            kModeRestart => self.timer_a_counter = self.timer_a_latch as i16,
            kModeOneTime => self.timer_a_enabled = false,
            _ => {}
        }
    }

    pub fn reset_timer_b(&mut self) {
        match self.timer_b_run_mode {
            kModeRestart => self.timer_b_counter = self.timer_b_latch as i16,
            kModeOneTime => self.timer_b_enabled = false,
            _ => {}
        }
    }

    pub fn vic_base_address(&self) -> u16 {
        ((!self.pra & 0x3) as u16) << 14
    }

    pub fn step(&mut self) -> bool {
        // Timer A
        if self.timer_a_enabled {
            match self.timer_a_input_mode {
                kModeProcessor => {
                    self.timer_a_counter -=
                        (self.cpu.borrow().cycles() - self.prev_cpu_cycles) as i16;
                    if self.timer_a_counter <= 0 {
                        if self.timer_a_irq_enabled {
                            self.timer_a_irq_triggered = true;
                            self.cpu.borrow_mut().nmi();
                        }
                        self.reset_timer_a();
                    }
                }
                kModeCNT => {}
            }
        }

        // Timer B
        if self.timer_b_enabled {
            match self.timer_b_input_mode {
                kModeProcessor => {
                    self.timer_b_counter -=
                        (self.cpu.borrow().cycles() - self.prev_cpu_cycles) as i16;
                    if self.timer_b_counter <= 0 {
                        if self.timer_b_irq_enabled {
                            self.timer_b_irq_triggered = true;
                            self.cpu.borrow_mut().nmi();
                        }
                        self.reset_timer_b();
                    }
                }
                kModeCNT => {}
                kModeTimerA => {}
                kModeTimerACNT => {}
            }
        }

        self.prev_cpu_cycles = self.cpu.borrow().cycles();

        true
    }
}
