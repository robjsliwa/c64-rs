use super::cpu::Cpu;
use super::memory::Memory;

pub struct Cia1 {
    cpu: Option<*mut Cpu>,
    memory: Option<*mut Memory>,
    timer_a_latch: i16,
    timer_b_latch: i16,
    timer_a_counter: i16,
    timer_b_counter: i16,
    timer_a_enabled: bool,
    timer_b_enabled: bool,
    timer_a_irq_enabled: bool,
    timer_b_irq_enabled: bool,
    timer_a_irq_triggered: bool,
    timer_b_irq_triggered: bool,
    timer_a_run_mode: RunMode,
    timer_b_run_mode: RunMode,
    timer_a_input_mode: InputMode,
    timer_b_input_mode: InputMode,
    prev_cpu_cycles: u32,
    pra: u8,
    prb: u8,
}

impl Cia1 {
    pub fn new() -> Self {
        Cia1 {
            cpu: None,
            memory: None,
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
            timer_a_run_mode: RunMode::Restart,
            timer_b_run_mode: RunMode::Restart,
            timer_a_input_mode: InputMode::Processor,
            timer_b_input_mode: InputMode::Processor,
            prev_cpu_cycles: 0,
            pra: 0xff,
            prb: 0xff,
        }
    }

    pub fn set_cpu(&mut self, cpu: &mut Cpu) {
        self.cpu = Some(cpu);
    }

    pub fn set_memory(&mut self, memory: &mut Memory) {
        self.memory = Some(memory);
    }

    pub fn write_register(&mut self, r: u8, v: u8) {
        match r {
            // data port a (PRA), keyboard matrix cols and joystick #2
            0x0 => {
                self.pra = v;
            }
            // data port b (PRB), keyboard matrix rows and joystick #1
            0x1 => {}
            // data direction port a (DDRA)
            0x2 => {}
            // data direction port b (DDRB)
            0x3 => {}
            // timer a low byte
            0x4 => {
                self.timer_a_latch &= 0xff00;
                self.timer_a_latch |= v as i16;
            }
            // timer a high byte
            0x5 => {
                self.timer_a_latch &= 0x00ff;
                self.timer_a_latch |= (v as i16) << 8;
            }
            // timer b low byte
            0x6 => {
                self.timer_b_latch &= 0xff00;
                self.timer_b_latch |= v as i16;
            }
            // timer b high byte
            0x7 => {
                self.timer_b_latch &= 0x00ff;
                self.timer_b_latch |= (v as i16) << 8;
            }
            _ => {}
        }
    }

    pub fn read_register(&self, r: u8) -> u8 {
        let mut retval = 0;
        match r {
            // data port a (PRA), keyboard matrix cols and joystick #2
            0x0 => {
                retval = self.pra;
            }
            // data port b (PRB), keyboard matrix rows and joystick #1
            0x1 => {}
            // timer control and status
            0xd => {
                if self.timer_a_irq_triggered || self.timer_b_irq_triggered {
                    retval |= 1 << 7; // IRQ occured
                    if self.timer_a_irq_triggered {
                        retval |= 1 << 0;
                    }
                    if self.timer_b_irq_triggered {
                        retval |= 1 << 1;
                    }
                }
            }
            _ => {}
        }
        retval
    }

    pub fn reset_timer_a(&mut self) {
        match self.timer_a_run_mode {
            RunMode::Restart => {
                self.timer_a_counter = self.timer_a_latch;
            }
            RunMode::OneTime => {
                self.timer_a_enabled = false;
            }
        }
    }

    pub fn reset_timer_b(&mut self) {
        match self.timer_b_run_mode {
            RunMode::Restart => {
                self.timer_b_counter = self.timer_b_latch;
            }
            RunMode::OneTime => {
                self.timer_b_enabled = false;
            }
        }
    }

    pub fn emulate(&mut self) -> bool {
        if let Some(cpu_ref) = self.cpu {
            let cpu = unsafe { &mut *cpu_ref };
            if self.timer_a_enabled {
                match self.timer_a_input_mode {
                    InputMode::Processor => {
                        self.timer_a_counter -= (cpu.cycles() - self.prev_cpu_cycles) as i16;
                        if self.timer_a_counter <= 0 {
                            if self.timer_a_irq_enabled {
                                self.timer_a_irq_triggered = true;
                                cpu.irq();
                            }
                            self.reset_timer_a();
                        }
                    }
                    _ => {}
                }
            }
            if self.timer_b_enabled {
                match self.timer_b_input_mode {
                    InputMode::Processor => {
                        self.timer_b_counter -= (cpu.cycles() - self.prev_cpu_cycles) as i16;
                        if self.timer_b_counter <= 0 {
                            if self.timer_b_irq_enabled {
                                self.timer_b_irq_triggered = true;
                                cpu.irq();
                            }
                            self.reset_timer_b();
                        }
                    }
                    _ => {}
                }
            }
            self.prev_cpu_cycles = cpu.cycles();
        }
        true
    }
}

// Constants (enums)
#[derive(Copy, Clone, PartialEq)]
enum InputMode {
    Processor,
    CNT,
    TimerA,
    TimerACNT,
}

#[derive(Copy, Clone, PartialEq)]
enum RunMode {
    Restart,
    OneTime,
}
