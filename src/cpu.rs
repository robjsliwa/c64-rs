use crate::memory::Memory;

pub struct CPU<'a> {
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

impl<'a> CPU<'a> {
    pub fn new(memory: &'a mut Memory) -> Self {
        CPU {
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
            0x01 => self.op_ora(self.addr_indirect_x()),
            // ... Add other opcodes here
            _ => panic!("Unknown opcode: {:02X}", opcode),
        }
    }

    // Addressing mode example
    fn addr_indirect_x(&self) -> u16 {
        // TODO: Implement indirect X addressing mode
        0 // Placeholder return value
    }

    /// Writes a byte to the memory the CPU is using
    pub fn write_memory(&mut self, addr: u16, value: u8) {
        self.memory.write_byte(addr, value);
    }

    /// Reads a byte from the memory the CPU is using
    pub fn read_memory(&self, addr: u16) -> u8 {
        self.memory.read_byte(addr)
    }

    // ---- Math Instructions ----
    // ADC: Add with Carry
    fn op_adc(&mut self, addr: u16) {
        let value = self.memory.read_byte(addr);
        let temp = self.a as u16 + value as u16 + if self.carry { 1 } else { 0 };

        self.overflow = (!(self.a ^ value) & (self.a ^ temp as u8) & 0x80) != 0;
        self.carry = temp > 0xFF;

        self.a = temp as u8;
        self.update_zero_negative_flags(self.a);
    }

    // SBC: Subtract with Carry
    fn op_sbc(&mut self, addr: u16) {
        let value = self.memory.read_byte(addr);
        let temp = self.a as i16 - value as i16 - if self.carry { 0 } else { 1 };

        self.overflow = ((self.a ^ temp as u8) & (self.a ^ value) & 0x80) != 0;
        self.carry = temp >= 0;

        self.a = temp as u8;
        self.update_zero_negative_flags(self.a);
    }

    // ---- Memory Instructions ----
    // LDA: Load Accumulator
    fn op_lda(&mut self, addr: u16) {
        self.a = self.memory.read_byte(addr);
        self.update_zero_negative_flags(self.a);
    }

    // LDX: Load X Register
    fn op_ldx(&mut self, addr: u16) {
        self.x = self.memory.read_byte(addr);
        self.update_zero_negative_flags(self.x);
    }

    // LDY: Load Y Register
    fn op_ldy(&mut self, addr: u16) {
        self.y = self.memory.read_byte(addr);
        self.update_zero_negative_flags(self.y);
    }

    // STA: Store Accumulator
    fn op_sta(&mut self, addr: u16) {
        self.memory.write_byte(addr, self.a);
    }

    // STX: Store X Register
    fn op_stx(&mut self, addr: u16) {
        self.memory.write_byte(addr, self.x);
    }

    // STY: Store Y Register
    fn op_sty(&mut self, addr: u16) {
        self.memory.write_byte(addr, self.y);
    }

    // INC: Increment Memory
    fn op_inc(&mut self, addr: u16) {
        let mut value = self.memory.read_byte(addr);
        value = value.wrapping_add(1);
        self.memory.write_byte(addr, value);
        self.update_zero_negative_flags(value);
    }

    // DEC: Decrement Memory
    fn op_dec(&mut self, addr: u16) {
        let mut value = self.memory.read_byte(addr);
        value = value.wrapping_sub(1);
        self.memory.write_byte(addr, value);
        self.update_zero_negative_flags(value);
    }

    // ---- Branching Instructions ----
    // BEQ: Branch if Equal (Zero flag is set)
    fn op_beq(&mut self, offset: i8) {
        if self.zero {
            self.branch(offset);
        }
    }

    // BNE: Branch if Not Equal (Zero flag is clear)
    fn op_bne(&mut self, offset: i8) {
        if !self.zero {
            self.branch(offset);
        }
    }

    // BCS: Branch if Carry Set
    fn op_bcs(&mut self, offset: i8) {
        if self.carry {
            self.branch(offset);
        }
    }

    // BCC: Branch if Carry Clear
    fn op_bcc(&mut self, offset: i8) {
        if !self.carry {
            self.branch(offset);
        }
    }

    // BMI: Branch if Minus (Negative flag is set)
    fn op_bmi(&mut self, offset: i8) {
        if self.negative {
            self.branch(offset);
        }
    }

    // BPL: Branch if Positive (Negative flag is clear)
    fn op_bpl(&mut self, offset: i8) {
        if !self.negative {
            self.branch(offset);
        }
    }

    // BVS: Branch if Overflow Set
    fn op_bvs(&mut self, offset: i8) {
        if self.overflow {
            self.branch(offset);
        }
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
    fn op_and(&mut self, addr: u16) {
        self.a &= self.memory.read_byte(addr);
        self.update_zero_negative_flags(self.a);
    }

    // ORA: Logical OR
    fn op_ora(&mut self, addr: u16) {
        self.a |= self.memory.read_byte(addr);
        self.update_zero_negative_flags(self.a);
    }

    // EOR: Exclusive OR
    fn op_eor(&mut self, addr: u16) {
        self.a ^= self.memory.read_byte(addr);
        self.update_zero_negative_flags(self.a);
    }

    // ASL: Arithmetic Shift Left
    fn op_asl(&mut self, addr: u16) {
        let mut value = self.memory.read_byte(addr);
        self.carry = (value & 0x80) != 0;
        value <<= 1;
        self.memory.write_byte(addr, value);
        self.update_zero_negative_flags(value);
    }

    // LSR: Logical Shift Right
    fn op_lsr(&mut self, addr: u16) {
        let mut value = self.memory.read_byte(addr);
        self.carry = (value & 0x01) != 0;
        value >>= 1;
        self.memory.write_byte(addr, value);
        self.update_zero_negative_flags(value);
    }

    // ROL: Rotate Left
    fn op_rol(&mut self, addr: u16) {
        let mut value = self.memory.read_byte(addr);
        let new_carry = (value & 0x80) != 0;
        value <<= 1;
        if self.carry {
            value |= 0x01;
        }
        self.memory.write_byte(addr, value);
        self.carry = new_carry;
        self.update_zero_negative_flags(value);
    }

    // ROR: Rotate Right
    fn op_ror(&mut self, addr: u16) {
        let mut value = self.memory.read_byte(addr);
        let new_carry = (value & 0x01) != 0;
        value >>= 1;
        if self.carry {
            value |= 0x80;
        }
        self.memory.write_byte(addr, value);
        self.carry = new_carry;
        self.update_zero_negative_flags(value);
    }

    // ---- Stack Instructions ----
    // PHA: Push Accumulator onto Stack
    fn op_pha(&mut self) {
        self.memory.write_byte(0x0100 + self.sp as u16, self.a);
        self.sp = self.sp.wrapping_sub(1);
    }

    // PHP: Push Processor Status onto Stack
    fn op_php(&mut self) {
        let status = self.status_from_flags();
        self.memory.write_byte(0x0100 + self.sp as u16, status);
        self.sp = self.sp.wrapping_sub(1);
    }

    // PLA: Pull Accumulator from Stack
    fn op_pla(&mut self) {
        self.sp = self.sp.wrapping_add(1);
        self.a = self.memory.read_byte(0x0100 + self.sp as u16);
        self.update_zero_negative_flags(self.a);
    }

    // PLP: Pull Processor Status from Stack
    fn op_plp(&mut self) {
        self.sp = self.sp.wrapping_add(1);
        let status = self.memory.read_byte(0x0100 + self.sp as u16);
        self.flags_from_status(status);
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

    // TODO:
    // - TSX: Transfer Stack Pointer to X Register
    // - TXS: Transfer X Register to Stack Pointer

    // ---- Jump Instructions ----
    // JMP: Jump to Address
    fn op_jmp(&mut self, addr: u16) {
        self.pc = addr;
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

    // TODO:
    // - RTI: Return from Interrupt

    // ---- Compare Instructions ----
    // CMP: Compare Accumulator
    fn op_cmp(&mut self, addr: u16) {
        let value = self.memory.read_byte(addr);
        let result = self.a.wrapping_sub(value);
        self.update_zero_negative_flags(result);
        self.carry = self.a >= value;
    }

    // CPX: Compare X Register
    fn op_cpx(&mut self, addr: u16) {
        let value = self.memory.read_byte(addr);
        let result = self.x.wrapping_sub(value);
        self.update_zero_negative_flags(result);
        self.carry = self.x >= value;
    }

    // CPY: Compare Y Register
    fn op_cpy(&mut self, addr: u16) {
        let value = self.memory.read_byte(addr);
        let result = self.y.wrapping_sub(value);
        self.update_zero_negative_flags(result);
        self.carry = self.y >= value;
    }

    // BIT: Bit Test
    fn op_bit(&mut self, addr: u16) {
        let value = self.memory.read_byte(addr);
        let result = self.a & value;

        self.zero = result == 0;
        self.overflow = (value & 0x40) != 0;
        self.negative = (value & 0x80) != 0;
    }

    // ---- Flag Instructions ----
    // CLC: Clear Carry Flag
    fn op_clc(&mut self) {
        self.carry = false;
    }

    // SEC: Set Carry Flag
    fn op_sec(&mut self) {
        self.carry = true;
    }

    // CLI: Clear Interrupt Disable Flag
    fn op_cli(&mut self) {
        self.interrupt_disable = false;
    }

    // SEI: Set Interrupt Disable Flag
    fn op_sei(&mut self) {
        self.interrupt_disable = true;
    }

    // CLV: Clear Overflow Flag
    fn op_clv(&mut self) {
        self.overflow = false;
    }

    // CLD: Clear Decimal Mode Flag
    fn op_cld(&mut self) {
        self.decimal = false;
    }

    // SED: Set Decimal Mode Flag
    fn op_sed(&mut self) {
        self.decimal = true;
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
    }

    // NOP: No Operation
    fn op_nop(&mut self) {
        // Do nothing
    }

    // Helper function to update the Zero and Negative flags
    fn update_zero_negative_flags(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = (value & 0x80) != 0;
    }
}
