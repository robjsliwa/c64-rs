use crate::common::is_bit_set;
use crate::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Cpu<'a> {
    pub pc: u16,                         // Program Counter
    pub sp: u8,                          // Stack Pointer
    pub a: u8,                           // Accumulator
    pub x: u8,                           // X register
    pub y: u8,                           // Y register
    pub memory: Rc<RefCell<Memory<'a>>>, // Reference to the memory
    cycles: u32,                         // CPU cycles

    // Flags
    cf: bool,
    zf: bool,
    idf: bool,
    dmf: bool,
    bcf: bool,
    of: bool,
    nf: bool,

    debug: bool,
}

impl<'a> Cpu<'a> {
    pub fn new(memory: Rc<RefCell<Memory<'a>>>) -> Self {
        Cpu {
            pc: 0,
            sp: 0xFF, // Stack starts at 0xFF
            a: 0,
            x: 0,
            y: 0,
            memory,
            cycles: 0,
            cf: false,
            zf: false,
            idf: true,
            dmf: false,
            bcf: false,
            of: false,
            nf: false,
            debug: false,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFF;
        self.cf = false;
        self.zf = false;
        self.idf = true;
        self.dmf = false;
        self.bcf = false;
        self.of = false;
        self.nf = false;
        self.pc = self.memory.borrow().read_word(0xFFFC); // Read reset vector
        self.cycles = 6;
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    fn print_memory(&self, addr: u16) -> String {
        let addr = addr - 1;
        format!(
            "Memory at {:#04X}: {:#04X} {:#04X} {:#04X} {:#04X}",
            addr,
            self.memory.borrow().read_byte(addr),
            self.memory.borrow().read_byte(addr + 1),
            self.memory.borrow().read_byte(addr + 2),
            self.memory.borrow().read_byte(addr + 3),
        )
    }

    fn disassemble(&mut self, opcode: u8) -> String {
        match opcode {
            // Add cases for each opcode
            0x00 => "BRK".to_string(),
            0x01 => format!("ORA ($44,X) -- {}", self.print_memory(self.pc)),
            0x05 => format!("ORA $44 -- {}", self.print_memory(self.pc)),
            0x06 => format!("ASL $44 -- {}", self.print_memory(self.pc)),
            0x08 => format!("PHP -- {}", self.print_memory(self.pc)),
            0x09 => format!("ORA #$44 -- {}", self.print_memory(self.pc)),
            0x0A => format!("ASL A -- {}", self.print_memory(self.pc)),
            0x0D => format!("ORA $4400 -- {}", self.print_memory(self.pc)),
            0x0E => format!("ASL $4400 -- {}", self.print_memory(self.pc)),
            0x10 => format!("BPL -- {}", self.print_memory(self.pc)),
            0x11 => format!("ORA ($44),Y -- {}", self.print_memory(self.pc)),
            0x15 => format!("ORA $44,X -- {}", self.print_memory(self.pc)),
            0x16 => format!("ASL $44,X -- {}", self.print_memory(self.pc)),
            0x18 => format!("CLC -- {}", self.print_memory(self.pc)),
            0x19 => format!("ORA $4400,Y -- {}", self.print_memory(self.pc)),
            0x1D => format!("ORA $4400,X -- {}", self.print_memory(self.pc)),
            0x1E => format!("ASL $4400,X -- {}", self.print_memory(self.pc)),
            0x20 => format!("JSR $5597 -- {}", self.print_memory(self.pc)),
            0x21 => format!("AND ($44,X) -- {}", self.print_memory(self.pc)),
            0x24 => format!("BIT $44 -- {}", self.print_memory(self.pc)),
            0x25 => format!("AND $44 -- {}", self.print_memory(self.pc)),
            0x26 => format!("ROL $44 -- {}", self.print_memory(self.pc)),
            0x28 => format!("PLP -- {}", self.print_memory(self.pc)),
            0x29 => format!("AND #$44 -- {}", self.print_memory(self.pc)),
            0x2A => format!("ROL A -- {}", self.print_memory(self.pc)),
            0x2C => format!("BIT $4400 -- {}", self.print_memory(self.pc)),
            0x2D => format!("AND $4400 -- {}", self.print_memory(self.pc)),
            0x2E => format!("ROL $4400 -- {}", self.print_memory(self.pc)),
            0x30 => format!("BMI -- {}", self.print_memory(self.pc)),
            0x31 => format!("AND ($44),Y -- {}", self.print_memory(self.pc)),
            0x35 => format!("AND $44,X -- {}", self.print_memory(self.pc)),
            0x36 => format!("ROL $44,X -- {}", self.print_memory(self.pc)),
            0x38 => format!("SEC -- {}", self.print_memory(self.pc)),
            0x39 => format!("AND $4400,Y -- {}", self.print_memory(self.pc)),
            0x3D => format!("AND $4400,X -- {}", self.print_memory(self.pc)),
            0x3E => format!("ROL $4400,X -- {}", self.print_memory(self.pc)),
            0x40 => format!("RTI -- {}", self.print_memory(self.pc)),
            0x41 => format!("EOR ($44,X) -- {}", self.print_memory(self.pc)),
            0x45 => format!("EOR $44 -- {}", self.print_memory(self.pc)),
            0x46 => format!("LSR $44 -- {}", self.print_memory(self.pc)),
            0x48 => format!("PHA -- {}", self.print_memory(self.pc)),
            0x49 => format!("EOR #$44 -- {}", self.print_memory(self.pc)),
            0x4A => format!("LSR A -- {}", self.print_memory(self.pc)),
            0x4C => format!("JMP $5597 -- {}", self.print_memory(self.pc)),
            0x4D => format!("EOR $4400 -- {}", self.print_memory(self.pc)),
            0x4E => format!("LSR $4400 -- {}", self.print_memory(self.pc)),
            0x50 => format!("BVC -- {}", self.print_memory(self.pc)),
            0x51 => format!("EOR ($44),Y -- {}", self.print_memory(self.pc)),
            0x55 => format!("EOR $44,X -- {}", self.print_memory(self.pc)),
            0x56 => format!("LSR $44,X -- {}", self.print_memory(self.pc)),
            0x58 => format!("CLI -- {}", self.print_memory(self.pc)),
            0x59 => format!("EOR $4400,Y -- {}", self.print_memory(self.pc)),
            0x5D => format!("EOR $4400,X -- {}", self.print_memory(self.pc)),
            0x5E => format!("LSR $4400,X -- {}", self.print_memory(self.pc)),
            0x60 => format!("RTS -- {}", self.print_memory(self.pc)),
            0x61 => format!("ADC ($44,X) -- {}", self.print_memory(self.pc)),
            0x65 => format!("ADC $44 -- {}", self.print_memory(self.pc)),
            0x66 => format!("ROR $44 -- {}", self.print_memory(self.pc)),
            0x68 => format!("PLA -- {}", self.print_memory(self.pc)),
            0x69 => format!("ADC #$44 -- {}", self.print_memory(self.pc)),
            0x6A => format!("ROR A -- {}", self.print_memory(self.pc)),
            0x6C => format!("JMP ($5597) -- {}", self.print_memory(self.pc)),
            0x6D => format!("ADC $4400 -- {}", self.print_memory(self.pc)),
            0x6E => format!("ROR $4400 -- {}", self.print_memory(self.pc)),
            0x70 => format!("BVS -- {}", self.print_memory(self.pc)),
            0x71 => format!("ADC ($44),Y -- {}", self.print_memory(self.pc)),
            0x75 => format!("ADC $44,X -- {}", self.print_memory(self.pc)),
            0x76 => format!("ROR $44,X -- {}", self.print_memory(self.pc)),
            0x78 => format!("SEI -- {}", self.print_memory(self.pc)),
            0x79 => format!("ADC $4400,Y -- {}", self.print_memory(self.pc)),
            0x7D => format!("ADC $4400,X -- {}", self.print_memory(self.pc)),
            0x7E => format!("ROR $4400,X -- {}", self.print_memory(self.pc)),
            0x81 => format!("STA ($44,X) -- {}", self.print_memory(self.pc)),
            0x84 => format!("STY $44 -- {}", self.print_memory(self.pc)),
            0x85 => format!("STA $44 -- {}", self.print_memory(self.pc)),
            0x86 => format!("STX $44 -- {}", self.print_memory(self.pc)),
            0x88 => format!("DEY -- {}", self.print_memory(self.pc)),
            0x8A => format!("TXA -- {}", self.print_memory(self.pc)),
            0x8C => format!("STY $4400 -- {}", self.print_memory(self.pc)),
            0x8D => format!("STA $4400 -- {}", self.print_memory(self.pc)),
            0x8E => format!("STX $4400 -- {}", self.print_memory(self.pc)),
            0x90 => format!("BCC -- {}", self.print_memory(self.pc)),
            0x91 => format!("STA ($44),Y -- {}", self.print_memory(self.pc)),
            0x94 => format!("STY $44,X -- {}", self.print_memory(self.pc)),
            0x95 => format!("STA $44,X -- {}", self.print_memory(self.pc)),
            0x96 => format!("STX $44,Y -- {}", self.print_memory(self.pc)),
            0x98 => format!("TYA -- {}", self.print_memory(self.pc)),
            0x99 => format!("STA $4400,Y -- {}", self.print_memory(self.pc)),
            0x9A => format!("TXS -- {}", self.print_memory(self.pc)),
            0x9D => format!("STA $4400,X -- {}", self.print_memory(self.pc)),
            0xA0 => format!("LDY #$44 -- {}", self.print_memory(self.pc)),
            0xA1 => format!("LDA ($44,X) -- {}", self.print_memory(self.pc)),
            0xA2 => format!("LDX #$44 -- {}", self.print_memory(self.pc)),
            0xA4 => format!("LDY $44 -- {}", self.print_memory(self.pc)),
            0xA5 => format!("LDA $44 -- {}", self.print_memory(self.pc)),
            0xA6 => format!("LDX $44 -- {}", self.print_memory(self.pc)),
            0xA8 => format!("TAY -- {}", self.print_memory(self.pc)),
            0xA9 => format!("LDA #$44 -- {}", self.print_memory(self.pc)),
            0xAA => format!("TAX -- {}", self.print_memory(self.pc)),
            0xAC => format!("LDY $4400 -- {}", self.print_memory(self.pc)),
            0xAD => format!("LDA $4400 -- {}", self.print_memory(self.pc)),
            0xAE => format!("LDX $4400 -- {}", self.print_memory(self.pc)),
            0xB0 => format!("BCS -- {}", self.print_memory(self.pc)),
            0xB1 => format!("LDA ($44),Y -- {}", self.print_memory(self.pc)),
            0xB4 => format!("LDY $44,X -- {}", self.print_memory(self.pc)),
            0xB5 => format!("LDA $44,X -- {}", self.print_memory(self.pc)),
            0xB6 => format!("LDX $44,Y -- {}", self.print_memory(self.pc)),
            0xB8 => format!("CLV -- {}", self.print_memory(self.pc)),
            0xB9 => format!("LDA $4400,Y -- {}", self.print_memory(self.pc)),
            0xBA => format!("TSX -- {}", self.print_memory(self.pc)),
            0xBC => format!("LDY $4400,X -- {}", self.print_memory(self.pc)),
            0xBD => format!("LDA $4400,X -- {}", self.print_memory(self.pc)),
            0xBE => format!("LDX $4400,Y -- {}", self.print_memory(self.pc)),
            0xC0 => format!("CPY #$44 -- {}", self.print_memory(self.pc)),
            0xC1 => format!("CMP ($44,X) -- {}", self.print_memory(self.pc)),
            0xC4 => format!("CPY $44 -- {}", self.print_memory(self.pc)),
            0xC5 => format!("CMP $44 -- {}", self.print_memory(self.pc)),
            0xC6 => format!("DEC $44 -- {}", self.print_memory(self.pc)),
            0xC8 => format!("INY -- {}", self.print_memory(self.pc)),
            0xC9 => format!("CMP #$44 -- {}", self.print_memory(self.pc)),
            0xCA => format!("DEX -- {}", self.print_memory(self.pc)),
            0xCC => format!("CPY $4400 -- {}", self.print_memory(self.pc)),
            0xCD => format!("CMP $4400 -- {}", self.print_memory(self.pc)),
            0xCE => format!("DEC $4400 -- {}", self.print_memory(self.pc)),
            0xD0 => format!("BNE -- {}", self.print_memory(self.pc)),
            0xD1 => format!("CMP ($44),Y -- {}", self.print_memory(self.pc)),
            0xD5 => format!("CMP $44,X -- {}", self.print_memory(self.pc)),
            0xD6 => format!("DEC $44,X -- {}", self.print_memory(self.pc)),
            0xD8 => format!("CLD -- {}", self.print_memory(self.pc)),
            0xD9 => format!("CMP $4400,Y -- {}", self.print_memory(self.pc)),
            0xDD => format!("CMP $4400,X -- {}", self.print_memory(self.pc)),
            0xDE => format!("DEC $4400,X -- {}", self.print_memory(self.pc)),
            0xE0 => format!("CPX #$44 -- {}", self.print_memory(self.pc)),
            0xE1 => format!("SBC ($44,X) -- {}", self.print_memory(self.pc)),
            0xE4 => format!("CPX $44 -- {}", self.print_memory(self.pc)),
            0xE5 => format!("SBC $44 -- {}", self.print_memory(self.pc)),
            0xE6 => format!("INC $44 -- {}", self.print_memory(self.pc)),
            0xE8 => format!("INX -- {}", self.print_memory(self.pc)),
            0xE9 => format!("SBC #$44 -- {}", self.print_memory(self.pc)),
            0xEA => format!("NOP -- {}", self.print_memory(self.pc)),
            0xEC => format!("CPX $4400 -- {}", self.print_memory(self.pc)),
            0xED => format!("SBC $4400 -- {}", self.print_memory(self.pc)),
            0xEE => format!("INC $4400 -- {}", self.print_memory(self.pc)),
            0xF0 => format!("BEQ -- {}", self.print_memory(self.pc)),
            0xF1 => format!("SBC ($44),Y -- {}", self.print_memory(self.pc)),
            0xF5 => format!("SBC $44,X -- {}", self.print_memory(self.pc)),
            0xF6 => format!("INC $44,X -- {}", self.print_memory(self.pc)),
            0xF8 => format!("SED -- {}", self.print_memory(self.pc)),
            0xF9 => format!("SBC $4400,Y -- {}", self.print_memory(self.pc)),
            0xFD => format!("SBC $4400,X -- {}", self.print_memory(self.pc)),
            0xFE => format!("INC $4400,X -- {}", self.print_memory(self.pc)),
            _ => format!("Unknown opcode: {:#04X}", opcode),
        }
    }

    pub fn step(&mut self) -> bool {
        let opcode = self.fetch_op();
        if self.debug {
            println!("{}", self.disassemble(opcode));
        }

        let mut retval = true;

        match opcode {
            0x00 => self.brk(),
            0x01 => {
                let addr = self.addr_indx();
                self.ora(self.load_byte(addr), 6)
            }
            0x05 => {
                let addr = self.addr_zero();
                let byte = self.load_byte(addr);
                self.ora(byte, 3)
            }
            0x06 => {
                let addr = self.addr_zero();
                self.asl_mem(addr, 5)
            }
            0x08 => self.php(),
            0x09 => {
                let byte = self.fetch_op();
                self.ora(byte, 2)
            }
            0x0A => self.asl_a(),
            0x0D => {
                let addr = self.addr_abs();
                let byte = self.load_byte(addr);
                self.ora(byte, 4)
            }
            0x0E => {
                let addr = self.addr_abs();
                self.asl_mem(addr, 6)
            }
            0x10 => self.bpl(),
            0x11 => {
                let addr = self.addr_indy();
                self.ora(self.load_byte(addr), 5)
            }
            0x15 => {
                let addr = self.addr_zerox();
                self.ora(self.load_byte(addr), 4)
            }
            0x16 => {
                let addr = self.addr_zerox();
                self.asl_mem(addr, 6)
            }
            0x18 => self.clc(),
            0x19 => {
                let addr = self.addr_absy();
                self.ora(self.load_byte(addr), 4)
            }
            0x1D => {
                let addr = self.addr_absx();
                self.ora(self.load_byte(addr), 4)
            }
            0x1E => {
                let addr = self.addr_absx();
                self.asl_mem(addr, 7)
            }
            0x20 => self.jsr(),
            0x21 => {
                let addr = self.addr_indx();
                self.and(self.load_byte(addr), 6)
            }
            0x24 => {
                let addr = self.addr_zero();
                self.bit(addr, 3)
            }
            0x25 => {
                let addr = self.addr_zero();
                self.and(self.load_byte(addr), 3)
            }
            0x26 => {
                let addr = self.addr_zero();
                self.rol_mem(addr, 5)
            }
            0x28 => self.plp(),
            0x29 => {
                let byte = self.fetch_op();
                self.and(byte, 2)
            }
            0x2A => self.rol_a(),
            0x2C => {
                let addr = self.addr_abs();
                self.bit(addr, 4)
            }
            0x2D => {
                let addr = self.addr_abs();
                self.and(self.load_byte(addr), 4)
            }
            0x2E => {
                let addr = self.addr_abs();
                self.rol_mem(addr, 6)
            }
            0x30 => self.bmi(),
            0x31 => {
                let addr = self.addr_indy();
                self.and(self.load_byte(addr), 5)
            }
            0x35 => {
                let addr = self.addr_zerox();
                self.and(self.load_byte(addr), 4)
            }
            0x36 => {
                let addr = self.addr_zerox();
                self.rol_mem(addr, 6)
            }
            0x38 => self.sec(),
            0x39 => {
                let addr = self.addr_absy();
                self.and(self.load_byte(addr), 4)
            }
            0x3D => {
                let addr = self.addr_absx();
                self.and(self.load_byte(addr), 4)
            }
            0x3E => {
                let addr = self.addr_absx();
                self.rol_mem(addr, 7)
            }
            0x40 => self.rti(),
            0x41 => {
                let addr = self.addr_indx();
                self.eor(self.load_byte(addr), 6)
            }
            0x45 => {
                let addr = self.addr_zero();
                self.eor(self.load_byte(addr), 3)
            }
            0x46 => {
                let addr = self.addr_zero();
                self.lsr_mem(addr, 5)
            }
            0x48 => self.pha(),
            0x49 => {
                let byte = self.fetch_op();
                self.eor(byte, 2)
            }
            0x4A => self.lsr_a(),
            0x4C => self.jmp(),
            0x4D => {
                let addr = self.addr_abs();
                self.eor(self.load_byte(addr), 4)
            }
            0x4E => {
                let addr = self.addr_abs();
                self.lsr_mem(addr, 6)
            }
            0x50 => self.bvc(),
            0x51 => {
                let addr = self.addr_indy();
                self.eor(self.load_byte(addr), 5)
            }
            0x55 => {
                let addr = self.addr_zerox();
                self.eor(self.load_byte(addr), 4)
            }
            0x56 => {
                let addr = self.addr_zerox();
                self.lsr_mem(addr, 6)
            }
            0x58 => self.cli(),
            0x59 => {
                let addr = self.addr_absy();
                self.eor(self.load_byte(addr), 4)
            }
            0x5D => {
                let addr = self.addr_absx();
                self.eor(self.load_byte(addr), 4)
            }
            0x5E => {
                let addr = self.addr_absx();
                self.lsr_mem(addr, 7)
            }
            0x60 => self.rts(),
            0x61 => {
                let addr = self.addr_indx();
                self.adc(self.load_byte(addr), 6)
            }
            0x65 => {
                let addr = self.addr_zero();
                self.adc(self.load_byte(addr), 3)
            }
            0x66 => {
                let addr = self.addr_zero();
                self.ror_mem(addr, 5)
            }
            0x68 => self.pla(),
            0x69 => {
                let byte = self.fetch_op();
                self.adc(byte, 2)
            }
            0x6A => self.ror_a(),
            0x6C => self.jmp_ind(),
            0x6D => {
                let addr = self.addr_abs();
                self.adc(self.load_byte(addr), 4)
            }
            0x6E => {
                let addr = self.addr_abs();
                self.ror_mem(addr, 6)
            }
            0x70 => self.bvs(),
            0x71 => {
                let addr = self.addr_indy();
                self.adc(self.load_byte(addr), 5)
            }
            0x75 => {
                let addr = self.addr_zerox();
                self.adc(self.load_byte(addr), 4)
            }
            0x76 => {
                let addr = self.addr_zerox();
                self.ror_mem(addr, 6)
            }
            0x78 => self.sei(),
            0x79 => {
                let addr = self.addr_absy();
                self.adc(self.load_byte(addr), 4)
            }
            0x7D => {
                let addr = self.addr_absx();
                self.adc(self.load_byte(addr), 4)
            }
            0x7E => {
                let addr = self.addr_absx();
                self.ror_mem(addr, 7)
            }
            0x81 => {
                let addr = self.addr_indx();
                self.sta(addr, 6)
            }
            0x84 => {
                let addr = self.addr_zero();
                self.sty(addr, 3)
            }
            0x85 => {
                let addr = self.addr_zero();
                self.sta(addr, 3)
            }
            0x86 => {
                let addr = self.addr_zero();
                self.stx(addr, 3)
            }
            0x88 => self.dey(),
            0x8A => self.txa(),
            0x8C => {
                let addr = self.addr_abs();
                self.sty(addr, 4)
            }
            0x8D => {
                let addr = self.addr_abs();
                self.sta(addr, 4)
            }
            0x8E => {
                let addr = self.addr_abs();
                self.stx(addr, 4)
            }
            0x90 => self.bcc(),
            0x91 => {
                let addr = self.addr_indy();
                self.sta(addr, 6)
            }
            0x94 => {
                let addr = self.addr_zerox();
                self.sty(addr, 4)
            }
            0x95 => {
                let addr = self.addr_zerox();
                self.sta(addr, 4)
            }
            0x96 => {
                let addr = self.addr_zeroy();
                self.stx(addr, 4)
            }
            0x98 => self.tya(),
            0x99 => {
                let addr = self.addr_absy();
                self.sta(addr, 5)
            }
            0x9A => self.txs(),
            0x9D => {
                let addr = self.addr_absx();
                self.sta(addr, 5)
            }
            0xA0 => {
                let byte = self.fetch_op();
                self.ldy(byte, 2)
            }
            0xA1 => {
                let addr = self.addr_indx();
                self.lda(self.load_byte(addr), 6)
            }
            0xA2 => {
                let byte = self.fetch_op();
                self.ldx(byte, 2)
            }
            0xA4 => {
                let addr = self.addr_zero();
                self.ldy(self.load_byte(addr), 3)
            }
            0xA5 => {
                let addr = self.addr_zero();
                self.lda(self.load_byte(addr), 3)
            }
            0xA6 => {
                let addr = self.addr_zero();
                self.ldx(self.load_byte(addr), 3)
            }
            0xA8 => self.tay(),
            0xA9 => {
                let byte = self.fetch_op();
                self.lda(byte, 2)
            }
            0xAA => self.tax(),
            0xAC => {
                let addr = self.addr_abs();
                self.ldy(self.load_byte(addr), 4)
            }
            0xAD => {
                let addr = self.addr_abs();
                self.lda(self.load_byte(addr), 4)
            }
            0xAE => {
                let addr = self.addr_abs();
                self.ldx(self.load_byte(addr), 4)
            }
            0xB0 => self.bcs(),
            0xB1 => {
                let addr = self.addr_indy();
                self.lda(self.load_byte(addr), 5)
            }
            0xB4 => {
                let addr = self.addr_zerox();
                self.ldy(self.load_byte(addr), 3)
            }
            0xB5 => {
                let addr = self.addr_zerox();
                self.lda(self.load_byte(addr), 3)
            }
            0xB6 => {
                let addr = self.addr_zeroy();
                self.ldx(self.load_byte(addr), 3)
            }
            0xB8 => self.clv(),
            0xB9 => {
                let addr = self.addr_absy();
                self.lda(self.load_byte(addr), 4)
            }
            0xBA => self.tsx(),
            0xBC => {
                let addr = self.addr_absx();
                self.ldy(self.load_byte(addr), 4)
            }
            0xBD => {
                let addr = self.addr_absx();
                self.lda(self.load_byte(addr), 4)
            }
            0xBE => {
                let addr = self.addr_absy();
                self.ldx(self.load_byte(addr), 4)
            }
            0xC0 => {
                let byte = self.fetch_op();
                self.cpy(byte, 2)
            }
            0xC1 => {
                let addr = self.addr_indx();
                self.cmp(self.load_byte(addr), 6)
            }
            0xC4 => {
                let addr = self.addr_zero();
                self.cpy(self.load_byte(addr), 3)
            }
            0xC5 => {
                let addr = self.addr_zero();
                self.cmp(self.load_byte(addr), 3)
            }
            0xC6 => {
                let addr = self.addr_zero();
                self.dec(addr, 5)
            }
            0xC8 => self.iny(),
            0xC9 => {
                let byte = self.fetch_op();
                self.cmp(byte, 2)
            }
            0xCA => self.dex(),
            0xCC => {
                let addr = self.addr_abs();
                self.cpy(self.load_byte(addr), 4)
            }
            0xCD => {
                let addr = self.addr_abs();
                self.cmp(self.load_byte(addr), 4)
            }
            0xCE => {
                let addr = self.addr_abs();
                self.dec(addr, 6)
            }
            0xD0 => self.bne(),
            0xD1 => {
                let addr = self.addr_indy();
                self.cmp(self.load_byte(addr), 5)
            }
            0xD5 => {
                let addr = self.addr_zerox();
                self.cmp(self.load_byte(addr), 4)
            }
            0xD6 => {
                let addr = self.addr_zerox();
                self.dec(addr, 6)
            }
            0xD8 => self.cld(),
            0xD9 => {
                let addr = self.addr_absy();
                self.cmp(self.load_byte(addr), 4)
            }
            0xDD => {
                let addr = self.addr_absx();
                self.cmp(self.load_byte(addr), 4)
            }
            0xDE => {
                let addr = self.addr_absx();
                self.dec(addr, 7)
            }
            0xE0 => {
                let byte = self.fetch_op();
                self.cpx(byte, 2)
            }
            0xE1 => {
                let addr = self.addr_indx();
                self.sbc(self.load_byte(addr), 6)
            }
            0xE4 => {
                let addr = self.addr_zero();
                self.cpx(self.load_byte(addr), 3)
            }
            0xE5 => {
                let addr = self.addr_zero();
                self.sbc(self.load_byte(addr), 3)
            }
            0xE6 => {
                let addr = self.addr_zero();
                self.inc(addr, 5)
            }
            0xE8 => self.inx(),
            0xE9 => {
                let byte = self.fetch_op();
                self.sbc(byte, 2)
            }
            0xEA => self.nop(),
            0xEC => {
                let addr = self.addr_abs();
                self.cpx(self.load_byte(addr), 4)
            }
            0xED => {
                let addr = self.addr_abs();
                self.sbc(self.load_byte(addr), 4)
            }
            0xEE => {
                let addr = self.addr_abs();
                self.inc(addr, 6)
            }
            0xF0 => self.beq(),
            0xF1 => {
                let addr = self.addr_indy();
                self.sbc(self.load_byte(addr), 5)
            }
            0xF5 => {
                let addr = self.addr_zerox();
                self.sbc(self.load_byte(addr), 4)
            }
            0xF6 => {
                let addr = self.addr_zerox();
                self.inc(addr, 6)
            }
            0xF8 => self.sed(),
            0xF9 => {
                let addr = self.addr_absy();
                self.sbc(self.load_byte(addr), 4)
            }
            0xFD => {
                let addr = self.addr_absx();
                self.sbc(self.load_byte(addr), 4)
            }
            0xFE => {
                let addr = self.addr_absx();
                self.inc(addr, 7)
            }
            _ => {
                println!("Unknown opcode: {:02X}", opcode);
                retval = false;
            }
        }
        retval
    }

    fn fetch_op(&mut self) -> u8 {
        let byte = self.load_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn load_byte(&self, addr: u16) -> u8 {
        self.memory.borrow().read_byte(addr)
    }

    fn push(&mut self, v: u8) {
        let addr = Memory::BASE_ADDR_STACK + self.sp as u16;
        self.memory.borrow_mut().write_byte(addr, v);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = self.sp as u16 + Memory::BASE_ADDR_STACK;
        self.load_byte(addr)
    }

    fn flags(&self) -> u8 {
        let mut v = 0;
        v |= self.cf as u8;
        v |= (self.zf as u8) << 1;
        v |= (self.idf as u8) << 2;
        v |= (self.dmf as u8) << 3;
        v |= 1 << 4; // brk & php instructions push the bcf flag active
        v |= 1 << 5; // unused, always set
        v |= (self.of as u8) << 6;
        v |= (self.nf as u8) << 7;
        v
    }

    fn set_flags(&mut self, v: u8) {
        self.cf = is_bit_set(v, 0);
        self.zf = is_bit_set(v, 1);
        self.idf = is_bit_set(v, 2);
        self.dmf = is_bit_set(v, 3);
        // self.bcf = is_bit_set(v, 4);
        self.of = is_bit_set(v, 6);
        self.nf = is_bit_set(v, 7);
    }

    fn tick(&mut self, v: u8) {
        self.cycles += v as u32;
    }

    fn addr_indx(&mut self) -> u16 {
        let addr_zero = self.addr_zero();
        let addr = self
            .memory
            .borrow()
            .read_word((addr_zero + self.x as u16) & 0xff);
        addr
    }

    fn addr_zero(&mut self) -> u16 {
        self.fetch_op() as u16
    }

    fn set_zf(&mut self, val: u8) {
        self.zf = val == 0;
    }

    fn set_nf(&mut self, val: u8) {
        self.nf = (val & 0x80) != 0;
    }

    fn fetch_opw(&mut self) -> u16 {
        let retval = self.memory.borrow().read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        retval
    }

    fn addr_indy(&mut self) -> u16 {
        let addr_zero = self.addr_zero();
        let addr = self.memory.borrow().read_word(addr_zero) + self.y as u16;
        addr
    }

    fn addr_zerox(&mut self) -> u16 {
        (self.fetch_op() as u16 + self.x as u16) & 0xff
    }

    fn addr_absy(&mut self) -> u16 {
        self.fetch_opw().wrapping_add(self.y as u16)
    }

    fn addr_absx(&mut self) -> u16 {
        self.fetch_opw().wrapping_add(self.x as u16)
    }

    // OP CODES
    fn brk(&mut self) {
        let pc = self.pc.wrapping_add(1);
        self.push((pc >> 8) as u8);
        self.push((pc & 0xff) as u8);
        self.push(self.flags());
        self.pc = self.memory.borrow().read_word(Memory::ADDR_IRQ_VECTOR);
        self.idf = true;
        self.bcf = true;
        self.tick(7);
    }

    fn ora(&mut self, v: u8, cycles: u8) {
        self.a |= v;
        self.set_zf(self.a);
        self.set_nf(self.a);
        self.tick(cycles);
    }

    fn asl_mem(&mut self, addr: u16, cycles: u8) {
        let v = self.load_byte(addr);
        self.memory.borrow_mut().write_byte(addr, v);
        let asl = self.asl(v);
        self.memory.borrow_mut().write_byte(addr, asl);
        self.tick(cycles);
    }

    fn asl(&mut self, v: u8) -> u8 {
        // let t = (v << 1) & 0xff;
        let t = v << 1;
        self.cf = (v & 0x80) != 0;
        self.zf = t == 0;
        self.nf = (t & 0x80) != 0;
        t
    }

    fn asl_a(&mut self) {
        self.a = self.asl(self.a);
        self.tick(2);
    }

    fn php(&mut self) {
        self.push(self.flags());
        self.tick(3);
    }

    fn addr_abs(&mut self) -> u16 {
        self.fetch_opw()
    }

    fn bpl(&mut self) {
        let addr = (self.fetch_op() as i8 as i16 + self.pc as i16) as u16;
        if !self.nf {
            self.pc = addr;
        }
        self.tick(2);
    }

    fn clc(&mut self) {
        self.cf = false;
        self.tick(2);
    }

    fn jsr(&mut self) {
        let addr = self.addr_abs();
        self.push(((self.pc.wrapping_sub(1)) >> 8) as u8); // TODO: Check this
        self.push(((self.pc.wrapping_sub(1)) & 0xff) as u8);
        self.pc = addr;
        self.tick(6);
    }

    fn and(&mut self, v: u8, cycles: u8) {
        self.a &= v;
        self.set_zf(self.a);
        self.set_nf(self.a);
        self.tick(cycles);
    }

    fn bit(&mut self, addr: u16, cycles: u8) {
        let t = self.load_byte(addr);
        self.of = (t & 0x40) != 0;
        self.set_nf(t);
        self.set_zf(t & self.a);
        self.tick(cycles);
    }

    fn rol(&mut self, v: u8) -> u8 {
        let t = ((v as u16) << 1) | (self.cf as u16);
        self.cf = (t & 0x100) != 0;
        let t = t as u8;
        self.set_zf(t);
        self.set_nf(t);
        t
    }

    fn rol_a(&mut self) {
        self.a = self.rol(self.a);
        self.tick(2);
    }

    fn rol_mem(&mut self, addr: u16, cycles: u8) {
        let v = self.load_byte(addr);
        self.memory.borrow_mut().write_byte(addr, v);
        let rol = self.rol(v);
        self.memory.borrow_mut().write_byte(addr, rol);
        self.tick(cycles);
    }

    fn plp(&mut self) {
        let pop = self.pop();
        self.set_flags(pop);
        self.tick(4);
    }

    fn bmi(&mut self) {
        let offset = self.fetch_op() as i8;
        let addr = (self.pc as i16).wrapping_add(offset as i16) as u16;
        if self.nf {
            self.pc = addr;
        }
        self.tick(2);
    }

    fn sec(&mut self) {
        self.cf = true;
        self.tick(2);
    }

    fn rti(&mut self) {
        let pop = self.pop();
        self.set_flags(pop);
        self.pc = self.pop() as u16 + ((self.pop() as u16) << 8);
        self.tick(7);
    }

    fn eor(&mut self, v: u8, cycles: u8) {
        self.a ^= v;
        self.set_zf(self.a);
        self.set_nf(self.a);
        self.tick(cycles);
    }

    fn lsr(&mut self, v: u8) -> u8 {
        let t = v >> 1;
        self.cf = (v & 0x1) != 0;
        self.set_zf(t);
        self.set_nf(t);
        t
    }

    fn lsr_a(&mut self) {
        self.a = self.lsr(self.a);
        self.tick(2);
    }

    fn lsr_mem(&mut self, addr: u16, cycles: u8) {
        let v = self.load_byte(addr);
        self.memory.borrow_mut().write_byte(addr, v);
        let lsr = self.lsr(v);
        self.memory.borrow_mut().write_byte(addr, lsr);
        self.tick(cycles);
    }

    fn pha(&mut self) {
        self.push(self.a);
        self.tick(3);
    }

    fn bvc(&mut self) {
        let offset = self.fetch_op() as i8;
        let addr = (self.pc as i16).wrapping_add(offset as i16) as u16;
        if !self.of {
            self.pc = addr;
        }
        self.tick(2);
    }

    fn bvs(&mut self) {
        let offset = self.fetch_op() as i8;
        let addr = (self.pc as i16).wrapping_add(offset as i16) as u16;
        if self.of {
            self.pc = addr;
        }
        self.tick(2);
    }

    fn jmp(&mut self) {
        self.pc = self.addr_abs();
        self.tick(3);
    }

    fn cli(&mut self) {
        self.idf = false;
        self.tick(2);
    }

    fn rts(&mut self) {
        let addr = (self.pop() as u16) + ((self.pop() as u16) << 8) + 1;
        self.pc = addr;
        self.tick(6);
    }

    fn adc(&mut self, v: u8, cycles: u8) {
        let mut t: u16;
        if self.dmf {
            t = (self.a as u16 & 0xf) + (v as u16 & 0xf) + (if self.cf { 1 } else { 0 });
            if t > 0x09 {
                t += 0x6;
            }
            t += (self.a as u16 & 0xf0) + (v as u16 & 0xf0);
            if (t & 0x1f0) > 0x90 {
                t += 0x60;
            }
        } else {
            t = self.a as u16 + v as u16 + (if self.cf { 1 } else { 0 });
        }
        self.cf = t > 0xff;
        t &= 0xff;
        self.of = !((self.a ^ v) & 0x80 != 0) && ((self.a ^ t as u8) & 0x80 != 0);
        self.set_zf(t.try_into().unwrap()); // TODO: Check this
        self.set_nf(t.try_into().unwrap());
        self.a = t as u8;
        self.tick(cycles); // TODO: Check this
    }

    fn ror(&mut self, v: u8) -> u8 {
        let t = (v >> 1) | ((self.cf as u8) << 7);
        self.cf = (v & 0x1) != 0;
        self.set_zf(t);
        self.set_nf(t);
        t
    }

    fn ror_a(&mut self) {
        self.a = self.ror(self.a);
        self.tick(2);
    }

    fn ror_mem(&mut self, addr: u16, cycles: u8) {
        let v = self.load_byte(addr);
        self.memory.borrow_mut().write_byte(addr, v);
        let ror = self.ror(v);
        self.memory.borrow_mut().write_byte(addr, ror);
        self.tick(cycles);
    }

    fn pla(&mut self) {
        self.a = self.pop();
        self.set_zf(self.a);
        self.set_nf(self.a);
        self.tick(4);
    }

    fn jmp_ind(&mut self) {
        let addr_abs = self.addr_abs();
        let addr = self.memory.borrow().read_word(addr_abs);
        self.pc = addr;
        self.tick(3);
    }

    fn sei(&mut self) {
        self.idf = true;
        self.tick(2);
    }

    fn sta(&mut self, addr: u16, cycles: u8) {
        self.memory.borrow_mut().write_byte(addr, self.a);
        self.tick(cycles);
    }

    fn stx(&mut self, addr: u16, cycles: u8) {
        self.memory.borrow_mut().write_byte(addr, self.x);
        self.tick(cycles);
    }

    fn sty(&mut self, addr: u16, cycles: u8) {
        self.memory.borrow_mut().write_byte(addr, self.y);
        self.tick(cycles);
    }

    fn txs(&mut self) {
        self.sp = self.x;
        self.tick(2);
    }

    fn tsx(&mut self) {
        self.x = self.sp;
        self.set_zf(self.x);
        self.set_nf(self.x);
        self.tick(2);
    }

    fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.set_zf(self.x);
        self.set_nf(self.x);
        self.tick(2);
    }

    fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.set_zf(self.y);
        self.set_nf(self.y);
        self.tick(2);
    }

    fn txa(&mut self) {
        self.a = self.x;
        self.set_zf(self.a);
        self.set_nf(self.a);
        self.tick(2);
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.set_zf(self.x);
        self.set_nf(self.x);
        self.tick(2);
    }

    fn tay(&mut self) {
        self.y = self.a;
        self.set_zf(self.y);
        self.set_nf(self.y);
        self.tick(2);
    }

    fn tya(&mut self) {
        self.a = self.y;
        self.set_zf(self.a);
        self.set_nf(self.a);
        self.tick(2);
    }

    fn bcc(&mut self) {
        let offset = self.fetch_op() as i8;
        let addr = (self.pc as i16).wrapping_add(offset as i16) as u16;
        if !self.cf {
            self.pc = addr;
        }
        self.tick(2);
    }

    fn addr_zeroy(&mut self) -> u16 {
        (self.fetch_op() as u16 + self.y as u16) & 0xff
    }

    fn lda(&mut self, v: u8, cycles: u8) {
        self.a = v;
        self.set_zf(self.a);
        self.set_nf(self.a);
        self.tick(cycles);
    }

    fn ldx(&mut self, v: u8, cycles: u8) {
        self.x = v;
        self.set_zf(self.x);
        self.set_nf(self.x);
        self.tick(cycles);
    }

    fn ldy(&mut self, v: u8, cycles: u8) {
        self.y = v;
        self.set_zf(self.y);
        self.set_nf(self.y);
        self.tick(cycles);
    }

    fn bcs(&mut self) {
        let offset = self.fetch_op() as i8;
        let addr = (self.pc as i16).wrapping_add(offset as i16) as u16;
        if self.cf {
            self.pc = addr;
        }
        self.tick(2);
    }

    fn clv(&mut self) {
        self.of = false;
        self.tick(2);
    }

    fn cmp(&mut self, v: u8, cycles: u8) {
        // let t = self.a as u16 - v as u16;
        let t = (self.a as u16).wrapping_sub(v as u16);
        self.cf = t < 0x100;
        let t = t as u8;
        self.set_zf(t);
        self.set_nf(t);
        self.tick(cycles);
    }

    fn cpx(&mut self, v: u8, cycles: u8) {
        // let t = self.x as u16 - v as u16;
        let t = (self.x as u16).wrapping_sub(v as u16);
        self.cf = t < 0x100;
        let t = t as u8;
        self.set_zf(t);
        self.set_nf(t);
        self.tick(cycles);
    }

    fn cpy(&mut self, v: u8, cycles: u8) {
        // let t = self.y as u16 - v as u16;
        let t = (self.y as u16).wrapping_sub(v as u16);
        self.cf = t < 0x100;
        let t = t as u8;
        self.set_zf(t);
        self.set_nf(t);
        self.tick(cycles);
    }

    fn dec(&mut self, addr: u16, cycles: u8) {
        let mut v = self.load_byte(addr);
        self.memory.borrow_mut().write_byte(addr, v);
        v = v.wrapping_sub(1);
        self.memory.borrow_mut().write_byte(addr, v);
        self.set_zf(v);
        self.set_nf(v);
        self.tick(cycles); // TODO: Check this
    }

    fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.set_zf(self.x);
        self.set_nf(self.x);
        self.tick(2);
    }

    fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.set_zf(self.y);
        self.set_nf(self.y);
        self.tick(2);
    }

    fn bne(&mut self) {
        let offset = self.fetch_op() as i8;
        let addr = (self.pc as i16).wrapping_add(offset as i16) as u16;
        if !self.zf {
            self.pc = addr;
        }
        self.tick(2);
    }

    fn cld(&mut self) {
        self.dmf = false;
        self.tick(2);
    }

    fn sbc(&mut self, v: u8, cycles: u8) {
        let mut t: u16;
        if self.dmf {
            // t = (self.a as u16 & 0xf) - (v as u16 & 0xf) - (if self.cf { 0 } else { 1 });
            t = (self.a as u16 & 0xf)
                .wrapping_sub(v as u16 & 0xf)
                .wrapping_sub(if self.cf { 0 } else { 1 });
            if (t & 0x10) != 0 {
                // t = ((t - 0x6) & 0xf) | ((self.a as u16 & 0xf0) - (v as u16 & 0xf0) - 0x10);
                t = ((t - 0x6) & 0xf)
                    .wrapping_add((self.a as u16 & 0xf0).wrapping_sub(v as u16 & 0xf0))
                    .wrapping_sub(0x10);
            } else {
                // t = (t & 0xf) | ((self.a as u16 & 0xf0) - (v as u16 & 0xf0));
                t = (t & 0xf).wrapping_add((self.a as u16 & 0xf0).wrapping_sub(v as u16 & 0xf0));
            }
            if (t & 0x100) != 0 {
                t -= 0x60;
            }
        } else {
            // t = self.a as u16 - v as u16 - (if self.cf { 0 } else { 1 });
            t = (self.a as u16)
                .wrapping_sub(v as u16)
                .wrapping_sub(if self.cf { 0 } else { 1 });
        }
        self.cf = t < 0x100;
        t &= 0xff;
        self.of = ((self.a ^ t as u8) & 0x80) != 0 && ((self.a ^ v) & 0x80) != 0;
        self.set_zf(t.try_into().unwrap()); // TODO: Check this
        self.set_nf(t.try_into().unwrap());
        self.a = t as u8;
        self.tick(cycles);
    }

    fn inc(&mut self, addr: u16, cycles: u8) {
        let mut v = self.load_byte(addr);
        self.memory.borrow_mut().write_byte(addr, v);
        v = v.wrapping_add(1);
        self.memory.borrow_mut().write_byte(addr, v);
        self.set_zf(v);
        self.set_nf(v);
        self.tick(cycles); // TODO: Check this
    }

    fn nop(&mut self) {
        self.tick(2);
    }

    fn beq(&mut self) {
        let offset = self.fetch_op() as i8;
        let addr = (self.pc as i16).wrapping_add(offset as i16) as u16;
        if self.zf {
            self.pc = addr;
        }
        self.tick(2);
    }

    fn sed(&mut self) {
        self.dmf = true;
        self.tick(2);
    }

    pub fn irq(&mut self) {
        if !self.idf {
            self.push((self.pc >> 8) as u8);
            self.push((self.pc & 0xff) as u8);

            self.push(self.flags() & 0xef);
            self.pc = self.memory.borrow().read_word(Memory::ADDR_IRQ_VECTOR);
            self.idf = true;
            self.tick(7);
        }
    }

    pub fn cycles(&self) -> u32 {
        self.cycles
    }

    pub fn nmi(&mut self) {
        self.push((self.pc >> 8) as u8);
        self.push((self.pc & 0xff) as u8);

        self.push(self.flags() & 0xef);
        self.pc = self.memory.borrow().read_word(Memory::ADDR_NMI_VECTOR);
        self.tick(7);
    }

    pub fn write_memory(&mut self, addr: u16, value: u8) {
        self.memory.borrow_mut().write_byte(addr, value);
    }

    pub fn read_memory(&self, addr: u16) -> u8 {
        self.memory.borrow().read_byte(addr)
    }
}
