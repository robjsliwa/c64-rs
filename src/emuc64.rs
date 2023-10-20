#[derive(Default)]
pub struct CPU {
    pub pc: u16,  // Program Counter
    pub sp: u8,   // Stack Pointer
    pub a: u8,    // Accumulator
    pub x: u8,    // X register
    pub y: u8,    // Y register
    pub status: u8, // Processor Status
}

impl CPU {
    pub fn new() -> Self {
        CPU { 
            pc: 0x0800,  // Starting address for the BASIC ROM
            sp: 0xFF, 
            a: 0, 
            x: 0, 
            y: 0, 
            status: 0x20  // The default status has the unused bit set
        }
    }

    pub fn read_byte(&self, mem: &Memory, addr: u16) -> u8 {
        mem.read_byte(addr)
    }

    pub fn write_byte(&mut self, mem: &mut Memory, addr: u16, val: u8) {
        mem.write_byte(addr, val);
    }

    pub fn push_stack(&mut self, mem: &mut Memory, val: u8) {
        let addr = 0x0100 + self.sp as u16;
        self.write_byte(mem, addr, val);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pull_stack(&mut self, mem: &Memory) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100 + self.sp as u16;
        self.read_byte(mem, addr)
    }

    pub fn step(&mut self, mem: &mut Memory) {
        let opcode = self.read_byte(mem, self.pc);
        self.pc += 1;

        match opcode {
            0xA9 => { // LDA Immediate
                let value = self.read_byte(mem, self.pc);
                self.a = value;
                self.pc += 1;
            },
            0xA5 => { // LDA ZeroPage
                let addr = self.read_byte(mem, self.pc) as u16;
                self.a = self.read_byte(mem, addr);
                self.pc += 1;
            },
            // ... Implement other opcodes ...

            _ => panic!("Unknown opcode: {:#02X}", opcode),
        }
    }
}

pub struct Memory {
    data: [u8; 65536],  // 64KB
}

impl Memory {
    pub fn new() -> Self {
        Memory { data: [0; 65536] }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }
}
