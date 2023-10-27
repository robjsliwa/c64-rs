use crate::memory::Memory;

pub struct Cpu<'a> {
    pub pc: u16,            // Program Counter
    pub sp: u8,             // Stack Pointer
    pub a: u8,              // Accumulator
    pub x: u8,              // X register
    pub y: u8,              // Y register
    status: u8,             // Processor Status
    memory: &'a mut Memory, // Reference to the memory
    cycles: u32,            // CPU cycles

    // Flags
    carry: bool,
    zero: bool,
    interrupt_disable: bool,
    decimal: bool,
    break_command: bool,
    overflow: bool,
    negative: bool,
}

impl<'a> Cpu<'a> {
    pub fn new(memory: &'a mut Memory) -> Self {
        Cpu {
            pc: 0,
            sp: 0xFF, // Stack starts at 0xFF
            a: 0,
            x: 0,
            y: 0,
            status: 0,
            memory,
            cycles: 0,
            carry: false,
            zero: false,
            interrupt_disable: true,
            decimal: false,
            break_command: false,
            overflow: false,
            negative: false,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFF;
        self.carry = false;
        self.zero = false;
        self.interrupt_disable = true;
        self.decimal = false;
        self.break_command = false;
        self.overflow = false;
        self.negative = false;
        self.pc = self.memory.read_word(0xFFFC); // Read reset vector
        self.cycles = 6;
    }

    pub fn step(&mut self) {
        let opcode = self.memory.read_byte(self.pc);
        self.pc += 1; // Increment PC after fetching the opcode

        match opcode {
            0x00 => self.op_brk(),
            0x01 => {
                let addr = self.addr_indx();
                let value = self.load_byte(addr);
                self.op_ora(value, 6)
            }
            0x05 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_ora(value, 3)
            }
            0x06 => {
                let addr = self.addr_zero();
                self.op_asl(addr, 5)
            }
            0x08 => self.op_php(),
            0x09 => {
                let addr = self.fetch_op();
                self.op_ora(addr.into(), 2);
            }
            0x0A => self.op_asl_a(),
            0x0D => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_ora(value, 4);
            }
            0x0E => {
                let addr = self.addr_abs();
                self.op_asl(addr, 6);
            }
            0x10 => self.op_bpl(),
            0x11 => {
                let addr = self.addr_indy();
                let value = self.load_byte(addr);
                self.op_ora(value, 5);
            }
            0x15 => {
                let addr = self.addr_zeroy();
                let value = self.load_byte(addr);
                self.op_ora(value, 4);
            }
            0x16 => {
                let addr = self.addr_zerox();
                self.op_asl(addr, 6);
            }
            0x18 => self.op_clc(),
            0x19 => {
                let addr = self.addr_absy();
                let value = self.load_byte(addr);
                self.op_ora(value, 4);
            }
            0x1D => {
                let addr = self.addr_absx();
                let value = self.load_byte(addr);
                self.op_ora(value, 4);
            }
            0x1E => {
                let addr = self.addr_absx();
                self.op_asl(addr, 7);
            }
            0x20 => {
                let addr = self.fetch_opw();
                self.op_jsr(addr);
            }
            0x21 => {
                let addr = self.addr_indx();
                let value = self.load_byte(addr);
                self.op_and(value, 6);
            }
            0x24 => {
                let addr = self.addr_zero();
                self.op_bit(addr, 3);
            }
            0x25 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_and(value, 3);
            }
            0x26 => {
                let addr = self.addr_zero();
                self.op_rol(addr, 5);
            }
            0x28 => self.op_plp(),
            0x29 => {
                let addr = self.fetch_op();
                self.op_and(addr.into(), 2);
            }
            0x2A => self.op_rol_a(),
            0x2C => {
                let addr = self.addr_abs();
                self.op_bit(addr, 4);
            }
            0x2D => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_and(value, 4);
            }
            0x2E => {
                let addr = self.addr_abs();
                self.op_rol(addr, 6);
            }
            0x30 => self.op_bmi(),
            0x31 => {
                let addr = self.addr_indy();
                let value = self.load_byte(addr);
                self.op_and(value, 5);
            }
            0x35 => {
                let addr = self.addr_zeroy();
                let value = self.load_byte(addr);
                self.op_and(value, 4);
            }
            0x36 => {
                let addr = self.addr_zeroy();
                self.op_rol(addr, 6);
            }
            0x38 => self.op_sec(),
            0x39 => {
                let addr = self.addr_absy();
                let value = self.load_byte(addr);
                self.op_and(value, 4);
            }
            0x3D => {
                let addr = self.addr_absx();
                let value = self.load_byte(addr);
                self.op_and(value, 4);
            }
            0x3E => {
                let addr = self.addr_absx();
                self.op_rol(addr, 7);
            }
            0x40 => self.op_rti(),
            0x41 => {
                let addr = self.addr_indx();
                let value = self.load_byte(addr);
                self.op_eor(value, 6);
            }
            0x45 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_eor(value, 3);
            }
            0x46 => {
                let addr = self.addr_zero();
                self.op_lsr(addr, 5);
            }
            0x48 => self.op_pha(),
            0x49 => {
                let addr = self.fetch_op();
                self.op_eor(addr, 2);
            }
            0x4A => self.op_lsr_a(),
            0x4C => self.op_jmp(),
            0x4D => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_eor(value, 4);
            }
            0x4E => {
                let addr = self.addr_abs();
                self.op_lsr(addr, 6);
            }
            0x50 => {
                let offset = self.fetch_op() as i8;
                self.op_bvc(offset);
            }
            0x51 => {
                let addr = self.addr_indy();
                let value = self.load_byte(addr);
                self.op_eor(value, 5);
            }
            0x55 => {
                let addr = self.addr_zerox();
                let value = self.load_byte(addr);
                self.op_eor(value, 4);
            }
            0x56 => {
                let addr = self.addr_zerox();
                self.op_lsr(addr, 6);
            }
            0x58 => self.op_cli(),
            0x59 => {
                let addr = self.addr_absy();
                let value = self.load_byte(addr);
                self.op_eor(value, 4);
            }
            0x5D => {
                let addr = self.addr_absx();
                let value = self.load_byte(addr);
                self.op_eor(value, 4);
            }
            0x5E => {
                let addr = self.addr_absx();
                self.op_lsr(addr, 7);
            }
            0x60 => self.op_rts(),
            0x61 => {
                let addr = self.addr_indx();
                let value = self.load_byte(addr);
                self.op_adc(value, 6);
            }
            0x65 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_adc(value, 3);
            }
            0x66 => {
                let addr = self.addr_zero();
                self.op_ror(addr, 5);
            }
            0x68 => self.op_pla(),
            0x69 => {
                let addr = self.fetch_op();
                self.op_adc(addr, 2);
            }
            0x6A => self.op_ror_a(),
            0x6C => self.op_jmp_ind(),
            0x6D => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_adc(value, 4);
            }
            0x6E => {
                let addr = self.addr_abs();
                self.op_ror(addr, 6);
            }
            0x70 => {
                let offset = self.fetch_op() as i8;
                self.op_bvs(offset);
            }
            0x71 => {
                let addr = self.addr_indy();
                let value = self.load_byte(addr);
                self.op_adc(value, 5);
            }
            0x75 => {
                let addr = self.addr_zerox();
                let value = self.load_byte(addr);
                self.op_adc(value, 4);
            }
            0x76 => {
                let addr = self.addr_zerox();
                self.op_ror(addr, 6);
            }
            0x78 => self.op_sei(),
            0x79 => {
                let addr = self.addr_absy();
                let value = self.load_byte(addr);
                self.op_adc(value, 4);
            }
            0x7D => {
                let addr = self.addr_absx();
                let value = self.load_byte(addr);
                self.op_adc(value, 4);
            }
            0x7E => {
                let addr = self.addr_absx();
                self.op_ror(addr, 7);
            }
            0x81 => {
                let addr = self.addr_indx();
                self.op_sta(addr, 6);
            }
            0x84 => {
                let addr = self.addr_zero();
                self.op_sty(addr, 3);
            }
            0x85 => {
                let addr = self.addr_zero();
                self.op_sta(addr, 3);
            }
            0x86 => {
                let addr = self.addr_zero();
                self.op_stx(addr, 3);
            }
            0x88 => self.op_dey(),
            0x8A => self.op_txa(),
            0x8C => {
                let addr = self.addr_abs();
                self.op_sty(addr, 4);
            }
            0x8D => {
                let addr = self.addr_abs();
                self.op_sta(addr, 4);
            }
            0x8E => {
                let addr = self.addr_abs();
                self.op_stx(addr, 4);
            }
            0x90 => {
                let offset = self.fetch_op() as i8;
                self.op_bcc(offset);
            }
            0x91 => {
                let addr = self.addr_indy();
                self.op_sta(addr, 6);
            }
            0x94 => {
                let addr = self.addr_zerox();
                self.op_sty(addr, 4);
            }
            0x95 => {
                let addr = self.addr_zerox();
                self.op_sta(addr, 4);
            }
            0x96 => {
                let addr = self.addr_zeroy();
                self.op_stx(addr, 4);
            }
            0x98 => self.op_tya(),
            0x99 => {
                let addr = self.addr_absy();
                self.op_sta(addr, 5);
            }
            0x9A => self.op_txs(),
            0x9D => {
                let addr = self.addr_absx();
                self.op_sta(addr, 5);
            }
            0xA0 => {
                let addr = self.fetch_op();
                self.op_ldy(addr, 2);
            }
            0xA1 => {
                let addr = self.addr_indx();
                let value = self.load_byte(addr);
                self.op_lda(value, 6);
            }
            0xA2 => {
                let addr = self.fetch_op();
                self.op_ldx(addr, 2);
            }
            0xA4 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_ldy(value, 3);
            }
            0xA5 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_lda(value, 3);
            }
            0xA6 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_ldx(value, 3);
            }
            0xA8 => self.op_tay(),
            0xA9 => {
                let addr = self.fetch_op();
                self.op_lda(addr.into(), 2);
            }
            0xAA => self.op_tax(),
            0xAC => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_ldy(value, 4);
            }
            0xAD => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_lda(value, 4);
            }
            0xAE => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_ldx(value, 4);
            }
            0xB0 => {
                let offset = self.fetch_op() as i8;
                self.op_bcs(offset);
            }
            0xB1 => {
                let addr = self.addr_indy();
                let value = self.load_byte(addr);
                self.op_lda(value, 5);
            }
            0xB4 => {
                let addr = self.addr_zerox();
                let value = self.load_byte(addr);
                self.op_ldy(value, 3);
            }
            0xB5 => {
                let addr = self.addr_zerox();
                let value = self.load_byte(addr);
                self.op_lda(value, 3);
            }
            0xB6 => {
                let addr = self.addr_zeroy();
                let value = self.load_byte(addr);
                self.op_ldx(value, 3);
            }
            0xB8 => self.op_clv(),
            0xB9 => {
                let addr = self.addr_absy();
                let value = self.load_byte(addr);
                self.op_lda(value, 4);
            }
            0xBA => self.op_tsx(),
            0xBC => {
                let addr = self.addr_absx();
                let value = self.load_byte(addr);
                self.op_ldy(value, 4);
            }
            0xBD => {
                let addr = self.addr_absx();
                let value = self.load_byte(addr);
                self.op_lda(value, 4);
            }
            0xBE => {
                let addr = self.addr_absy();
                let value = self.load_byte(addr);
                self.op_ldx(value, 4);
            }
            0xC0 => {
                let addr = self.fetch_op();
                self.op_cpy(addr, 2);
            }
            0xC1 => {
                let addr = self.addr_indx();
                let value = self.load_byte(addr);
                self.op_cmp(value, 6);
            }
            0xC4 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_cpy(value, 3);
            }
            0xC5 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_cmp(value, 3);
            }
            0xC6 => {
                let addr = self.addr_zero();
                self.op_dec(addr, 5);
            }
            0xC8 => self.op_iny(),
            0xC9 => {
                let addr = self.fetch_op();
                self.op_cmp(addr, 2);
            }
            0xCA => self.op_dex(),
            0xCC => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_cpy(value, 4);
            }
            0xCD => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_cmp(value, 4);
            }
            0xCE => {
                let addr = self.addr_abs();
                self.op_dec(addr, 6);
            }
            0xD0 => {
                let offset = self.fetch_op() as i8;
                self.op_bne(offset);
            }
            0xD1 => {
                let addr = self.addr_indy();
                let value = self.load_byte(addr);
                self.op_cmp(value, 5);
            }
            0xD5 => {
                let addr = self.addr_zerox();
                let value = self.load_byte(addr);
                self.op_cmp(value, 4);
            }
            0xD6 => {
                let addr = self.addr_zerox();
                self.op_dec(addr, 6);
            }
            0xD8 => self.op_cld(),
            0xD9 => {
                let addr = self.addr_absy();
                let value = self.load_byte(addr);
                self.op_cmp(value, 4);
            }
            0xDD => {
                let addr = self.addr_absx();
                let value = self.load_byte(addr);
                self.op_cmp(value, 4);
            }
            0xDE => {
                let addr = self.addr_absx();
                self.op_dec(addr, 7);
            }
            0xE0 => {
                let addr = self.fetch_op();
                self.op_cpx(addr.into(), 2);
            }
            0xE1 => {
                let addr = self.addr_indx();
                let value = self.load_byte(addr);
                self.op_sbc(value, 6);
            }
            0xE4 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_cpx(value, 3);
            }
            0xE5 => {
                let addr = self.addr_zero();
                let value = self.load_byte(addr);
                self.op_sbc(value, 3);
            }
            0xE6 => {
                let addr = self.addr_zero();
                self.op_inc(addr, 5);
            }
            0xE8 => self.op_inx(),
            0xE9 => {
                let addr = self.fetch_op();
                self.op_sbc(addr, 2);
            }
            0xEA => self.op_nop(),
            0xEC => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_cpx(value, 4);
            }
            0xED => {
                let addr = self.addr_abs();
                let value = self.load_byte(addr);
                self.op_sbc(value, 4);
            }
            0xEE => {
                let addr = self.addr_abs();
                self.op_inc(addr, 6);
            }
            0xF0 => {
                let offset = self.fetch_op() as i8;
                self.op_beq(offset);
            }
            0xF1 => {
                let addr = self.addr_indy();
                let value = self.load_byte(addr);
                self.op_sbc(value, 5);
            }
            0xF5 => {
                let addr = self.addr_zerox();
                let value = self.load_byte(addr);
                self.op_sbc(value, 4);
            }
            0xF6 => {
                let addr = self.addr_zerox();
                self.op_inc(addr, 6);
            }
            0xF8 => self.op_sed(),
            0xF9 => {
                let addr = self.addr_absy();
                let value = self.load_byte(addr);
                self.op_sbc(value, 4);
            }
            0xFD => {
                let addr = self.addr_absx();
                let value = self.load_byte(addr);
                self.op_sbc(value, 4);
            }
            0xFE => {
                let addr = self.addr_absx();
                self.op_inc(addr, 7);
            }
            _ => panic!("Unknown opcode: {:02X}", opcode),
        }
    }

    // ---- Helper Functions ----
    pub fn cycles(&self) -> u32 {
        self.cycles
    }

    pub fn irq(&mut self) {
        // Push the current program counter onto the stack
        self.push_word(self.pc);

        // Push the processor status onto the stack
        self.status_from_flags();
        self.push(self.status);

        // Set the IRQ disable flag
        self.interrupt_disable = true;

        // Load the program counter with the address from the IRQ vector
        let lo = self.memory.read_byte(Memory::ADDR_IRQ_VECTOR) as u16;
        let hi = self.memory.read_byte(Memory::ADDR_IRQ_VECTOR + 1) as u16;
        self.pc = (hi << 8) | lo;
    }

    pub fn load_byte(&self, addr: u16) -> u8 {
        self.memory.read_byte(addr)
    }

    pub fn push(&mut self, v: u8) {
        let addr = Memory::BASE_ADDR_STACK + self.sp as u16;
        self.memory.write_byte(addr, v);
        self.sp -= 1;
    }

    pub fn pop(&mut self) -> u8 {
        let addr = (self.sp + 1) as u16 + Memory::BASE_ADDR_STACK;
        self.sp += 1;
        self.load_byte(addr)
    }

    pub fn fetch_op(&mut self) -> u8 {
        let opcode = self.load_byte(self.pc);
        self.pc += 1;
        opcode
    }

    pub fn fetch_opw(&mut self) -> u16 {
        let retval = self.memory.read_word(self.pc);
        self.pc += 2;
        retval
    }

    pub fn addr_zero(&mut self) -> u16 {
        self.fetch_op() as u16
    }

    pub fn addr_zerox(&mut self) -> u16 {
        (self.fetch_op() as u16 + self.x as u16) & 0xff
    }

    pub fn addr_zeroy(&mut self) -> u16 {
        (self.fetch_op() as u16 + self.y as u16) & 0xff
    }

    pub fn addr_abs(&mut self) -> u16 {
        self.fetch_opw()
    }

    pub fn addr_absy(&mut self) -> u16 {
        self.fetch_opw() + self.y as u16
    }

    pub fn addr_absx(&mut self) -> u16 {
        self.fetch_opw() + self.x as u16
    }

    pub fn addr_indx(&mut self) -> u16 {
        let addr = (self.addr_zero() + self.x as u16) & 0xff;
        self.memory.read_word(addr)
    }

    pub fn addr_indy(&mut self) -> u16 {
        let addr = self.addr_zero();
        self.memory.read_word(addr) + self.y as u16
    }

    // Advenced cycle count
    fn tick(&mut self, cycles: u32) {
        self.cycles += cycles;
    }

    /// Writes a byte to the memory the CPU is using
    pub fn write_memory(&mut self, addr: u16, value: u8) {
        self.memory.write_byte(addr, value);
    }

    /// Reads a byte from the memory the CPU is using
    pub fn read_memory(&self, addr: u16) -> u8 {
        self.memory.read_byte(addr)
    }

    /// Read a word from the memory the CPU is using
    /// The 6502 is little endian, so the first byte is the LSB
    /// and the second byte is the MSB
    fn read_word(&self, addr: u16) -> u16 {
        let lsb = self.read_memory(addr) as u16;
        let msb = self.read_memory(addr + 1) as u16;
        lsb | (msb << 8)
    }

    fn write_word(&mut self, addr: u16, value: u16) {
        let lsb = value as u8;
        let msb = (value >> 8) as u8;
        self.write_memory(addr, lsb);
        self.write_memory(addr + 1, msb);
    }

    // ---- Math Instructions ----
    // ADC: Add with Carry
    fn op_adc(&mut self, value: u8, cycles: u32) {
        let temp = self.a as u16 + value as u16 + if self.carry { 1 } else { 0 };

        self.overflow = (!(self.a ^ value) & (self.a ^ temp as u8) & 0x80) != 0;
        self.carry = temp > 0xFF;

        self.a = temp as u8;
        self.update_zero_negative_flags(self.a);
        self.tick(cycles);
    }

    // SBC: Subtract with Carry
    fn op_sbc(&mut self, value: u8, cycles: u32) {
        let temp = self.a as i16 - value as i16 - if self.carry { 0 } else { 1 };

        self.overflow = ((self.a ^ temp as u8) & (self.a ^ value) & 0x80) != 0;
        self.carry = temp >= 0;

        self.a = temp as u8;
        self.update_zero_negative_flags(self.a);
        self.tick(cycles);
    }

    // ---- Memory Instructions ----
    // LDA: Load Accumulator
    fn op_lda(&mut self, value: u8, cycles: u32) {
        self.a = value;
        self.update_zero_negative_flags(self.a);
        self.tick(cycles);
    }

    // LDX: Load X Register
    fn op_ldx(&mut self, value: u8, cycles: u32) {
        self.x = value;
        self.update_zero_negative_flags(self.x);
        self.tick(cycles);
    }

    // LDY: Load Y Register
    fn op_ldy(&mut self, value: u8, cycles: u32) {
        self.y = value;
        self.update_zero_negative_flags(self.y);
        self.tick(cycles);
    }

    // STA: Store Accumulator
    fn op_sta(&mut self, addr: u16, cycles: u32) {
        self.memory.write_byte(addr, self.a);
        self.tick(cycles);
    }

    // INC: Increment Memory
    fn op_inc(&mut self, addr: u16, cycles: u32) {
        let mut value = self.memory.read_byte(addr);
        value = value.wrapping_add(1);
        self.memory.write_byte(addr, value);
        self.update_zero_negative_flags(value);
        self.tick(cycles);
    }

    // DEC: Decrement Memory
    fn op_dec(&mut self, addr: u16, cycles: u32) {
        let mut value = self.memory.read_byte(addr);
        value = value.wrapping_sub(1);
        self.memory.write_byte(addr, value);
        self.update_zero_negative_flags(value);
        self.tick(cycles);
    }

    // ---- Branching Instructions ----
    // BEQ: Branch if Equal (Zero flag is set)
    fn op_beq(&mut self, offset: i8) {
        if self.zero {
            self.branch(offset);
        }
        self.tick(2);
    }

    // BNE: Branch if Not Equal (Zero flag is clear)
    fn op_bne(&mut self, offset: i8) {
        if !self.zero {
            self.branch(offset);
        }
        self.tick(2);
    }

    // BCS: Branch if Carry Set
    fn op_bcs(&mut self, offset: i8) {
        if self.carry {
            self.branch(offset);
        }
        self.tick(2);
    }

    // BCC: Branch if Carry Clear
    fn op_bcc(&mut self, offset: i8) {
        if !self.carry {
            self.branch(offset);
        }
        self.tick(2);
    }

    // BMI: Branch if Minus (Negative flag is set)
    fn op_bmi(&mut self) {
        let offset = self.fetch_op() + self.pc as u8;
        if self.negative {
            self.branch(offset as i8);
        }
        self.tick(2);
    }

    // BPL: Branch if Positive (Negative flag is clear)
    fn op_bpl(&mut self) {
        let offset = self.fetch_op() + self.pc as u8;
        if !self.negative {
            self.branch(offset as i8);
        }
        self.tick(2);
    }

    // BVS: Branch if Overflow Set
    fn op_bvs(&mut self, offset: i8) {
        if self.overflow {
            self.branch(offset);
        }
        self.tick(2);
    }

    // BVC: Branch if Overflow Clear
    fn op_bvc(&mut self, offset: i8) {
        if !self.overflow {
            self.branch(offset);
        }
    }

    // Helper function to handle branching
    fn branch(&mut self, offset: i8) {
        self.pc = (self.pc as i16 + offset as i16) as u16;
    }

    // ---- Bitwise Instructions ----
    // AND: Logical AND
    fn op_and(&mut self, value: u8, cycles: u32) {
        self.a &= value;
        self.update_zero_negative_flags(self.a);
        self.tick(cycles);
    }

    // ORA: Logical OR
    fn op_ora(&mut self, value: u8, cycles: u32) {
        self.a |= value;
        self.update_zero_negative_flags(self.a);
        self.tick(cycles);
    }

    // EOR: Exclusive OR
    fn op_eor(&mut self, value: u8, cycles: u32) {
        self.a ^= value;
        self.update_zero_negative_flags(self.a);
        self.tick(cycles);
    }

    // ASL: Arithmetic Shift Left
    fn op_asl(&mut self, addr: u16, cycles: u32) {
        let mut value = self.memory.read_byte(addr);
        self.carry = (value & 0x80) != 0;
        value <<= 1;
        self.memory.write_byte(addr, value);
        self.update_zero_negative_flags(value);
        self.tick(cycles);
    }

    fn op_asl_a(&mut self) {
        self.carry = (self.a & 0x80) != 0;
        self.a <<= 1;
        self.update_zero_negative_flags(self.a);
        self.tick(2);
    }

    // LSR: Logical Shift Right
    fn op_lsr(&mut self, addr: u16, cycles: u32) {
        let mut value = self.memory.read_byte(addr);
        self.carry = (value & 0x01) != 0;
        value >>= 1;
        self.memory.write_byte(addr, value);
        self.update_zero_negative_flags(value);
        self.tick(cycles);
    }

    fn op_lsr_a(&mut self) {
        self.carry = (self.a & 0x01) != 0;
        self.a >>= 1;
        self.update_zero_negative_flags(self.a);
        self.tick(2);
    }

    // ROL: Rotate Left
    fn op_rol(&mut self, addr: u16, cycles: u32) {
        let mut value = self.memory.read_byte(addr);
        let new_carry = (value & 0x80) != 0;
        value <<= 1;
        if self.carry {
            value |= 0x01;
        }
        self.memory.write_byte(addr, value);
        self.carry = new_carry;
        self.update_zero_negative_flags(value);
        self.tick(cycles);
    }

    fn op_rol_a(&mut self) {
        let new_carry = (self.a & 0x80) != 0;
        self.a <<= 1;
        if self.carry {
            self.a |= 0x01;
        }
        self.carry = new_carry;
        self.update_zero_negative_flags(self.a);
        self.tick(2);
    }

    // ROR: Rotate Right
    fn op_ror(&mut self, addr: u16, cycles: u32) {
        let mut value = self.memory.read_byte(addr);
        let new_carry = (value & 0x01) != 0;
        value >>= 1;
        if self.carry {
            value |= 0x80;
        }
        self.memory.write_byte(addr, value);
        self.carry = new_carry;
        self.update_zero_negative_flags(value);
        self.tick(cycles);
    }

    fn op_ror_a(&mut self) {
        let new_carry = (self.a & 0x01) != 0;
        self.a >>= 1;
        if self.carry {
            self.a |= 0x80;
        }
        self.carry = new_carry;
        self.update_zero_negative_flags(self.a);
        self.tick(2);
    }

    // ---- Stack Instructions ----
    // PHA: Push Accumulator onto Stack
    fn op_pha(&mut self) {
        self.memory.write_byte(0x0100 + self.sp as u16, self.a);
        self.sp = self.sp.wrapping_sub(1);
        self.tick(3);
    }

    // PHP: Push Processor Status onto Stack
    fn op_php(&mut self) {
        let status = self.status_from_flags();
        self.memory.write_byte(0x0100 + self.sp as u16, status);
        self.sp = self.sp.wrapping_sub(1);
        self.tick(3);
    }

    // PLA: Pull Accumulator from Stack
    fn op_pla(&mut self) {
        self.sp = self.sp.wrapping_add(1);
        self.a = self.memory.read_byte(0x0100 + self.sp as u16);
        self.update_zero_negative_flags(self.a);
        self.tick(4);
    }

    // PLP: Pull Processor Status from Stack
    fn op_plp(&mut self) {
        self.sp = self.sp.wrapping_add(1);
        let status = self.memory.read_byte(0x0100 + self.sp as u16);
        self.flags_from_status(status);
        self.tick(4);
    }

    // STX: Store X Register
    fn op_stx(&mut self, addr: u16, cycles: u32) {
        self.memory.write_byte(addr, self.x);
        self.tick(cycles);
    }

    // STY: Store Y Register
    fn op_sty(&mut self, addr: u16, cycles: u32) {
        self.memory.write_byte(addr, self.y);
        self.tick(cycles);
    }

    // TXS: Transfer X to Stack Pointer
    fn op_txs(&mut self) {
        self.sp = self.x;
        self.tick(2);
    }

    // TSX: Transfer Stack Pointer to X
    fn op_tsx(&mut self) {
        self.x = self.sp;
        self.update_zero_negative_flags(self.x);
        self.tick(2);
    }

    // Helper functions to convert between status flags and a single byte
    fn status_from_flags(&self) -> u8 {
        let mut status = 0;
        if self.carry {
            status |= 1 << 0;
        }
        if self.zero {
            status |= 1 << 1;
        }
        if self.interrupt_disable {
            status |= 1 << 2;
        }
        if self.decimal {
            status |= 1 << 3;
        }
        if self.break_command {
            status |= 1 << 4;
        }
        // bit 5 is always 1
        status |= 1 << 5;
        if self.overflow {
            status |= 1 << 6;
        }
        if self.negative {
            status |= 1 << 7;
        }
        status
    }

    fn flags_from_status(&mut self, status: u8) {
        self.carry = (status & (1 << 0)) != 0;
        self.zero = (status & (1 << 1)) != 0;
        self.interrupt_disable = (status & (1 << 2)) != 0;
        self.decimal = (status & (1 << 3)) != 0;
        self.break_command = (status & (1 << 4)) != 0;
        self.overflow = (status & (1 << 6)) != 0;
        self.negative = (status & (1 << 7)) != 0;
    }

    // -- Register Instructions --
    fn op_tax(&mut self) {
        self.x = self.a;
        self.update_zero_negative_flags(self.x);
        self.tick(2);
    }

    fn op_tay(&mut self) {
        self.y = self.a;
        self.update_zero_negative_flags(self.y);
        self.tick(2);
    }

    fn op_txa(&mut self) {
        self.a = self.x;
        self.update_zero_negative_flags(self.a);
        self.tick(2);
    }

    fn op_tya(&mut self) {
        self.a = self.y;
        self.update_zero_negative_flags(self.a);
        self.tick(2);
    }

    fn op_dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.update_zero_negative_flags(self.x);
        self.tick(2);
    }

    fn op_dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.update_zero_negative_flags(self.y);
        self.tick(2);
    }

    fn op_inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.update_zero_negative_flags(self.x);
        self.tick(2);
    }

    fn op_iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.update_zero_negative_flags(self.y);
        self.tick(2);
    }

    // ---- Jump Instructions ----
    // JMP: Jump to Address
    fn op_jmp(&mut self) {
        let addr = self.addr_abs();
        self.pc = addr;
        self.tick(3);
    }

    fn op_jmp_ind(&mut self) {
        let addr = self.addr_abs();
        let value = self.read_word(addr);
        self.pc = value;
        self.tick(3);
    }

    // JSR: Jump to Subroutine
    fn op_jsr(&mut self, addr: u16) {
        // Push the return address (minus one) onto the stack
        self.push_word(self.pc.wrapping_sub(1));
        self.pc = addr;
    }

    // RTS: Return from Subroutine
    fn op_rts(&mut self) {
        self.pc = self.pull_word().wrapping_add(1);
        self.tick(6);
    }

    // RTI: Return from Interrupt
    fn op_rti(&mut self) {
        self.op_plp();
        self.pc = self.pull_word();
        self.tick(7);
    }

    // Helper functions for stack operations
    fn push_word(&mut self, value: u16) {
        let hi = ((value >> 8) & 0xFF) as u8;
        let lo = (value & 0xFF) as u8;
        self.memory.write_byte(0x0100 + self.sp as u16, hi);
        self.sp = self.sp.wrapping_sub(1);
        self.memory.write_byte(0x0100 + self.sp as u16, lo);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pull_word(&mut self) -> u16 {
        self.sp = self.sp.wrapping_add(1);
        let lo = self.memory.read_byte(0x0100 + self.sp as u16) as u16;
        self.sp = self.sp.wrapping_add(1);
        let hi = self.memory.read_byte(0x0100 + self.sp as u16) as u16;
        (hi << 8) | lo
    }

    // ---- Compare Instructions ----
    // CMP: Compare Accumulator
    fn op_cmp(&mut self, value: u8, cycles: u32) {
        let result = self.a.wrapping_sub(value);
        self.update_zero_negative_flags(result);
        self.carry = self.a >= value;
        self.tick(cycles);
    }

    // CPX: Compare X Register
    fn op_cpx(&mut self, value: u8, cycles: u32) {
        let result = self.x.wrapping_sub(value);
        self.update_zero_negative_flags(result);
        self.carry = self.x >= value;
        self.tick(cycles);
    }

    // CPY: Compare Y Register
    fn op_cpy(&mut self, value: u8, cycles: u32) {
        let result = self.y.wrapping_sub(value);
        self.update_zero_negative_flags(result);
        self.carry = self.y >= value;
        self.tick(cycles);
    }

    // BIT: Bit Test
    fn op_bit(&mut self, addr: u16, cycles: u32) {
        let value = self.memory.read_byte(addr);
        let result = self.a & value;

        self.zero = result == 0;
        self.overflow = (value & 0x40) != 0;
        self.negative = (value & 0x80) != 0;
        self.tick(cycles);
    }

    // ---- Flag Instructions ----
    // CLC: Clear Carry Flag
    fn op_clc(&mut self) {
        self.carry = false;
    }

    // SEC: Set Carry Flag
    fn op_sec(&mut self) {
        self.carry = true;
        self.tick(2);
    }

    // CLI: Clear Interrupt Disable Flag
    fn op_cli(&mut self) {
        self.interrupt_disable = false;
        self.tick(2);
    }

    // SEI: Set Interrupt Disable Flag
    fn op_sei(&mut self) {
        self.interrupt_disable = true;
        self.tick(2);
    }

    // CLV: Clear Overflow Flag
    fn op_clv(&mut self) {
        self.overflow = false;
        self.tick(2);
    }

    // CLD: Clear Decimal Mode Flag
    fn op_cld(&mut self) {
        self.decimal = false;
        self.tick(2);
    }

    // SED: Set Decimal Mode Flag
    fn op_sed(&mut self) {
        self.decimal = true;
        self.tick(2);
    }

    // ---- Other Instructions ----
    // BRK: Break
    fn op_brk(&mut self) {
        // Push program counter to stack
        self.push_word(self.pc);

        // Push status register to stack
        self.op_php();

        // Set interrupt disable flag to prevent further interrupts
        self.interrupt_disable = true;

        // Load interrupt vector into program counter
        self.pc = self.memory.read_word(0xFFFE);

        self.tick(7);
    }

    // NOP: No Operation
    fn op_nop(&mut self) {
        self.tick(2);
    }

    // Helper function to update the Zero and Negative flags
    fn update_zero_negative_flags(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = (value & 0x80) != 0;
    }
}
