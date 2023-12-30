pub enum InputMode {
    Processor,
    CNT,
    TimerA,
    TimerACNT,
}

impl InputMode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
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

pub enum RunMode {
    Restart,
    OneTime,
}

impl RunMode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Option<RunMode> {
        match value {
            0 => Some(RunMode::Restart),
            1 => Some(RunMode::OneTime),
            _ => None,
        }
    }
}
