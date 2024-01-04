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

    pub fn step(&mut self) -> bool {
        let opcode = self.fetch_op();
        let mut retval = true;

        match opcode {
            0x00 => self.brk(),
            0x01 => self.ora(self.load_byte(self.addr_indx()), 6),
            0x05 => {
                let addr = self.addr_zero();
                let byte = self.load_byte(addr);
                self.ora(byte, 3)
            }
            0x06 => self.asl_mem(self.addr_zero(), 5),
            0x08 => self.php(),
            0x09 => self.ora(self.fetch_op(), 2),
            0x0A => self.asl_a(),
            0x0D => self.ora(self.load_byte(self.addr_abs()), 4),
            0x0E => self.asl_mem(self.addr_abs(), 6),
            0x10 => self.bpl(),
            0x11 => self.ora(self.load_byte(self.addr_indy()), 5),
            0x15 => self.ora(self.load_byte(self.addr_zerox()), 4),
            0x16 => self.asl_mem(self.addr_zerox(), 6),
            0x18 => self.clc(),
            0x19 => self.ora(self.load_byte(self.addr_absy()), 4),
            0x1D => self.ora(self.load_byte(self.addr_absx()), 4),
            0x1E => self.asl_mem(self.addr_absx(), 7),
            0x20 => self.jsr(),
            0x21 => self.and(self.load_byte(self.addr_indx()), 6),
            0x24 => self.bit(self.addr_zero(), 3),
            0x25 => self.and(self.load_byte(self.addr_zero()), 3),
            0x26 => self.rol_mem(self.addr_zero(), 5),
            0x28 => self.plp(),
            0x29 => self.and(self.fetch_op(), 2),
            0x2A => self.rol_a(),
            0x2C => self.bit(self.addr_abs(), 4),
            0x2D => self.and(self.load_byte(self.addr_abs()), 4),
            0x2E => self.rol_mem(self.addr_abs(), 6),
            0x30 => self.bmi(),
            0x31 => self.and(self.load_byte(self.addr_indy()), 5),
            0x35 => self.and(self.load_byte(self.addr_zerox()), 4),
            0x36 => self.rol_mem(self.addr_zerox(), 6),
            0x38 => self.sec(),
            0x39 => self.and(self.load_byte(self.addr_absy()), 4),
            0x3D => self.and(self.load_byte(self.addr_absx()), 4),
            0x3E => self.rol_mem(self.addr_absx(), 7),
            0x40 => self.rti(),
            0x41 => self.eor(self.load_byte(self.addr_indx()), 6),
            0x45 => self.eor(self.load_byte(self.addr_zero()), 3),
            0x46 => self.lsr_mem(self.addr_zero(), 5),
            0x48 => self.pha(),
            0x49 => self.eor(self.fetch_op(), 2),
            0x4A => self.lsr_a(),
            0x4C => self.jmp(),
            0x4D => self.eor(self.load_byte(self.addr_abs()), 4),
            0x4E => self.lsr_mem(self.addr_abs(), 6),
            0x50 => self.bvc(),
            0x51 => self.eor(self.load_byte(self.addr_indy()), 5),
            0x55 => self.eor(self.load_byte(self.addr_zerox()), 4),
            0x56 => self.lsr_mem(self.addr_zerox(), 6),
            0x58 => self.cli(),
            0x59 => self.eor(self.load_byte(self.addr_absy()), 4),
            0x5D => self.eor(self.load_byte(self.addr_absx()), 4),
            0x5E => self.lsr_mem(self.addr_absx(), 7),
            0x60 => self.rts(),
            0x61 => self.adc(self.load_byte(self.addr_indx()), 6),
            0x65 => self.adc(self.load_byte(self.addr_zero()), 3),
            0x66 => self.ror_mem(self.addr_zero(), 5),
            0x68 => self.pla(),
            0x69 => self.adc(self.fetch_op(), 2),
            0x6A => self.ror_a(),
            0x6C => self.jmp_ind(),
            0x6D => self.adc(self.load_byte(self.addr_abs()), 4),
            0x6E => self.ror_mem(self.addr_abs(), 6),
            0x70 => self.bvs(),
            0x71 => self.adc(self.load_byte(self.addr_indy()), 5),
            0x75 => self.adc(self.load_byte(self.addr_zerox()), 4),
            0x76 => self.ror_mem(self.addr_zerox(), 6),
            0x78 => self.sei(),
            0x79 => self.adc(self.load_byte(self.addr_absy()), 4),
            0x7D => self.adc(self.load_byte(self.addr_absx()), 4),
            0x7E => self.ror_mem(self.addr_absx(), 7),
            0x81 => self.sta(self.addr_indx(), 6),
            0x84 => self.sty(self.addr_zero(), 3),
            0x85 => self.sta(self.addr_zero(), 3),
            0x86 => self.stx(self.addr_zero(), 3),
            0x88 => self.dey(),
            0x8A => self.txa(),
            0x8C => self.sty(self.addr_abs(), 4),
            0x8D => self.sta(self.addr_abs(), 4),
            0x8E => self.stx(self.addr_abs(), 4),
            0x90 => self.bcc(),
            0x91 => self.sta(self.addr_indy(), 6),
            0x94 => self.sty(self.addr_zerox(), 4),
            0x95 => self.sta(self.addr_zerox(), 4),
            0x96 => self.stx(self.addr_zeroy(), 4),
            0x98 => self.tya(),
            0x99 => self.sta(self.addr_absy(), 5),
            0x9A => self.txs(),
            0x9D => self.sta(self.addr_absx(), 5),
            0xA0 => self.ldy(self.fetch_op(), 2),
            0xA1 => self.lda(self.load_byte(self.addr_indx()), 6),
            0xA2 => self.ldx(self.fetch_op(), 2),
            0xA4 => self.ldy(self.load_byte(self.addr_zero()), 3),
            0xA5 => self.lda(self.load_byte(self.addr_zero()), 3),
            0xA6 => self.ldx(self.load_byte(self.addr_zero()), 3),
            0xA8 => self.tay(),
            0xA9 => self.lda(self.fetch_op(), 2),
            0xAA => self.tax(),
            0xAC => self.ldy(self.load_byte(self.addr_abs()), 4),
            0xAD => self.lda(self.load_byte(self.addr_abs()), 4),
            0xAE => self.ldx(self.load_byte(self.addr_abs()), 4),
            0xB0 => self.bcs(),
            0xB1 => self.lda(self.load_byte(self.addr_indy()), 5),
            0xB4 => self.ldy(self.load_byte(self.addr_zerox()), 3),
            0xB5 => self.lda(self.load_byte(self.addr_zerox()), 3),
            0xB6 => self.ldx(self.load_byte(self.addr_zeroy()), 3),
            0xB8 => self.clv(),
            0xB9 => self.lda(self.load_byte(self.addr_absy()), 4),
            0xBA => self.tsx(),
            0xBC => self.ldy(self.load_byte(self.addr_absx()), 4),
            0xBD => self.lda(self.load_byte(self.addr_absx()), 4),
            0xBE => self.ldx(self.load_byte(self.addr_absy()), 4),
            0xC0 => self.cpy(self.fetch_op(), 2),
            0xC1 => self.cmp(self.load_byte(self.addr_indx()), 6),
            0xC4 => self.cpy(self.load_byte(self.addr_zero()), 3),
            0xC5 => self.cmp(self.load_byte(self.addr_zero()), 3),
            0xC6 => self.dec(self.addr_zero(), 5),
            0xC8 => self.iny(),
            0xC9 => self.cmp(self.fetch_op(), 2),
            0xCA => self.dex(),
            0xCC => self.cpy(self.load_byte(self.addr_abs()), 4),
            0xCD => self.cmp(self.load_byte(self.addr_abs()), 4),
            0xCE => self.dec(self.addr_abs(), 6),
            0xD0 => self.bne(),
            0xD1 => self.cmp(self.load_byte(self.addr_indy()), 5),
            0xD5 => self.cmp(self.load_byte(self.addr_zerox()), 4),
            0xD6 => self.dec(self.addr_zerox(), 6),
            0xD8 => self.cld(),
            0xD9 => self.cmp(self.load_byte(self.addr_absy()), 4),
            0xDD => self.cmp(self.load_byte(self.addr_absx()), 4),
            0xDE => self.dec(self.addr_absx(), 7),
            0xE0 => self.cpx(self.fetch_op(), 2),
            0xE1 => self.sbc(self.load_byte(self.addr_indx()), 6),
            0xE4 => self.cpx(self.load_byte(self.addr_zero()), 3),
            0xE5 => self.sbc(self.load_byte(self.addr_zero()), 3),
            0xE6 => self.inc(self.addr_zero(), 5),
            0xE8 => self.inx(),
            0xE9 => self.sbc(self.fetch_op(), 2),
            0xEA => self.nop(),
            0xEC => self.cpx(self.load_byte(self.addr_abs()), 4),
            0xED => self.sbc(self.load_byte(self.addr_abs()), 4),
            0xEE => self.inc(self.addr_abs(), 6),
            0xF0 => self.beq(),
            0xF1 => self.sbc(self.load_byte(self.addr_indy()), 5),
            0xF5 => self.sbc(self.load_byte(self.addr_zerox()), 4),
            0xF6 => self.inc(self.addr_zerox(), 6),
            0xF8 => self.sed(),
            0xF9 => self.sbc(self.load_byte(self.addr_absy()), 4),
            0xFD => self.sbc(self.load_byte(self.addr_absx()), 4),
            0xFE => self.inc(self.addr_absx(), 7),
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

    fn addr_indx(&self) -> u16 {
        let addr = self
            .memory
            .borrow()
            .read_word((self.addr_zero() + self.x as u16) & 0xff);
        addr
    }

    fn addr_zero(&mut self) -> u16 {
        let addr = self.fetch_op() as u16;
        addr
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

    fn addr_indy(&self) -> u16 {
        let addr = self.memory.borrow().read_word(self.addr_zero()) + self.y as u16;
        addr
    }

    fn addr_zerox(&mut self) -> u16 {
        let addr = (self.fetch_op() as u16 + self.x as u16) & 0xff;
        addr
    }

    fn addr_absy(&mut self) -> u16 {
        let addr = self.fetch_opw().wrapping_add(self.y as u16);
        addr
    }

    fn addr_absx(&mut self) -> u16 {
        let addr = self.fetch_opw().wrapping_add(self.x as u16);
        addr
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
        self.memory.borrow_mut().write_byte(addr, self.asl(v));
        self.tick(cycles);
    }

    fn asl(&mut self, v: u8) -> u8 {
        let t = (v << 1) & 0xff;
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
        let addr = self.fetch_opw();
        addr
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
        self.set_zf(t.try_into().unwrap()); // TODO: Check this
        self.set_nf(t.try_into().unwrap());
        t as u8
    }

    fn rol_a(&mut self) {
        self.a = self.rol(self.a);
        self.tick(2);
    }

    fn rol_mem(&mut self, addr: u16, cycles: u8) {
        let v = self.load_byte(addr);
        self.memory.borrow_mut().write_byte(addr, v);
        self.memory.borrow_mut().write_byte(addr, self.rol(v));
        self.tick(cycles);
    }

    fn plp(&mut self) {
        self.set_flags(self.pop());
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
        self.set_flags(self.pop());
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
        self.memory.borrow_mut().write_byte(addr, self.lsr(v));
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
        self.memory.borrow_mut().write_byte(addr, self.ror(v));
        self.tick(cycles);
    }

    fn pla(&mut self) {
        self.a = self.pop();
        self.set_zf(self.a);
        self.set_nf(self.a);
        self.tick(4);
    }

    fn jmp_ind(&mut self) {
        let addr = self.memory.borrow().read_word(self.addr_abs());
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
        let addr = (self.fetch_op() as u16 + self.y as u16) & 0xff;
        addr
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
        let t = self.a as u16 - v as u16;
        self.cf = t < 0x100;
        let t = t as u8;
        self.set_zf(t);
        self.set_nf(t);
        self.tick(cycles);
    }

    fn cpx(&mut self, v: u8, cycles: u8) {
        let t = self.x as u16 - v as u16;
        self.cf = t < 0x100;
        let t = t as u8;
        self.set_zf(t);
        self.set_nf(t);
        self.tick(cycles);
    }

    fn cpy(&mut self, v: u8, cycles: u8) {
        let t = self.y as u16 - v as u16;
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
            t = (self.a as u16 & 0xf) - (v as u16 & 0xf) - (if self.cf { 0 } else { 1 });
            if (t & 0x10) != 0 {
                t = ((t - 0x6) & 0xf) | ((self.a as u16 & 0xf0) - (v as u16 & 0xf0) - 0x10);
            } else {
                t = (t & 0xf) | ((self.a as u16 & 0xf0) - (v as u16 & 0xf0));
            }
            if (t & 0x100) != 0 {
                t -= 0x60;
            }
        } else {
            t = self.a as u16 - v as u16 - (if self.cf { 0 } else { 1 });
        }
        self.cf = t < 0x100;
        t = t & 0xff;
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
