use super::cpu::Cpu;

pub struct Cia1<'a> {
    cpu: &'a mut Cpu<'a>,
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
    timer_a_run_mode: RunMode,
    timer_b_run_mode: RunMode,
    timer_a_input_mode: InputMode,
    timer_b_input_mode: InputMode,
    prev_cpu_cycles: u32,
    pra: u8,
    prb: u8,
}

impl<'a> Cia1<'a> {
    pub fn new(cpu: &'a mut Cpu<'a>) -> Self {
        Cia1 {
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
            timer_a_run_mode: RunMode::Restart,
            timer_b_run_mode: RunMode::Restart,
            timer_a_input_mode: InputMode::Processor,
            timer_b_input_mode: InputMode::Processor,
            prev_cpu_cycles: 0,
            pra: 0xff,
            prb: 0xff,
        }
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
                self.timer_a_latch |= v as u16;
            }
            // timer a high byte
            0x5 => {
                self.timer_a_latch &= 0x00ff;
                self.timer_a_latch |= (v as u16) << 8;
            }
            // timer b low byte
            0x6 => {
                self.timer_b_latch &= 0xff00;
                self.timer_b_latch |= v as u16;
            }
            // timer b high byte
            0x7 => {
                self.timer_b_latch &= 0x00ff;
                self.timer_b_latch |= (v as u16) << 8;
            }
            // RTC 1/10s
            0x8 => {}
            /* RTC seconds */
            0x9 => {}
            /* RTC minutes */
            0xa => {}
            /* RTC hours */
            0xb => {}
            /* shift serial */
            0xc => {}
            /* interrupt control and status */
            0xd => {
                // if bit 7 is set, enable selected mask of
                // interrupts, else disable them
                if (v & (1 << 7)) != 0 {
                    self.timer_a_irq_enabled = (v & (1 << 0)) != 0;
                    self.timer_b_irq_enabled = (v & (1 << 1)) != 0;
                } else {
                    self.timer_a_irq_enabled = false;
                    self.timer_b_irq_enabled = false;
                }
            }
            // control timer a
            0xe => {
                self.timer_a_enabled = (v & 0x1) != 0;
                self.timer_a_input_mode = InputMode::from((v & (1 << 5)) >> 5);
                // load latch requested
                if (v & (1 << 4)) != 0 {
                    self.timer_a_counter = self.timer_a_latch as i16;
                }
            }
            // control timer b
            0xf => {
                self.timer_b_enabled = (v & 0x1) != 0;
                self.timer_b_input_mode = InputMode::from((v & (1 << 5)) >> 5);
                // load latch requested
                if (v & (1 << 4)) != 0 {
                    self.timer_b_counter = self.timer_b_latch as i16;
                }
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
            0x1 => {
                if self.pra == 0xff {
                    retval = 0xff;
                } else if self.pra != 0 {
                    let col = 0;
                    let v: u8 = !self.pra;
                    while {
                        v >>= 1;
                        v != 0
                    } {
                        col += 1;
                    }

                    retval = self.io.keyboard_matrix(col);
                }
            }
            // data direction port a (DDRA)
            0x2 => {}
            // data direction port b (DDRB)
            0x3 => {}
            // timer a low byte
            0x4 => {
                retval = (self.timer_a_counter & 0x00ff) as u8;
            }
            // timer a high byte
            0x5 => {
                retval = ((self.timer_a_counter & 0xff00) >> 8) as u8;
            }
            // timer b low byte
            0x6 => {
                retval = (self.timer_b_counter & 0x00ff) as u8;
            }
            // timer b high byte
            0x7 => {
                retval = ((self.timer_b_counter & 0xff00) >> 8) as u8;
            }
            // RTC 1/10s
            0x8 => {}
            // RTC seconds
            0x9 => {}
            // RTC minutes
            0xa => {}
            // RTC hours
            0xb => {}
            // shift serial
            0xc => {}
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
            // control timer a
            0xe => {}
            // control timer b
            0xf => {}
            _ => {}
        }
        retval
    }

    pub fn reset_timer_a(&mut self) {
        match self.timer_a_run_mode {
            RunMode::Restart => {
                self.timer_a_counter = self.timer_a_latch as i16;
            }
            RunMode::OneTime => {
                self.timer_a_enabled = false;
            }
        }
    }

    pub fn reset_timer_b(&mut self) {
        match self.timer_b_run_mode {
            RunMode::Restart => {
                self.timer_b_counter = self.timer_b_latch as i16;
            }
            RunMode::OneTime => {
                self.timer_b_enabled = false;
            }
        }
    }

    pub fn emulate(&mut self) -> bool {
        if self.timer_a_enabled {
            match self.timer_a_input_mode {
                InputMode::Processor => {
                    self.timer_a_counter -= (self.cpu.cycles() - self.prev_cpu_cycles) as i16;
                    if self.timer_a_counter <= 0 {
                        if self.timer_a_irq_enabled {
                            self.timer_a_irq_triggered = true;
                            self.cpu.irq();
                        }
                        self.reset_timer_a();
                    }
                }
                InputMode::CNT => {}
                InputMode::TimerA => {}
                InputMode::TimerACNT => {}
            }
        }
        if self.timer_b_enabled {
            match self.timer_b_input_mode {
                InputMode::Processor => {
                    self.timer_b_counter -= (self.cpu.cycles() - self.prev_cpu_cycles) as i16;
                    if self.timer_b_counter <= 0 {
                        if self.timer_b_irq_enabled {
                            self.timer_b_irq_triggered = true;
                            self.cpu.irq();
                        }
                        self.reset_timer_b();
                    }
                }
                InputMode::CNT => {}
                InputMode::TimerA => {}
                InputMode::TimerACNT => {}
            }
        }
        self.prev_cpu_cycles = self.cpu.cycles();
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

impl From<u8> for InputMode {
    fn from(value: u8) -> Self {
        match value {
            0 => InputMode::Processor,
            1 => InputMode::CNT,
            2 => InputMode::TimerA,
            3 => InputMode::TimerACNT,
            _ => panic!("Invalid value for InputMode: {}", value),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum RunMode {
    Restart,
    OneTime,
}
