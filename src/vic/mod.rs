pub mod colors;
pub mod constants;
pub mod registers;

use crate::cpu::Cpu;
use crate::io::IO;
use crate::memory::Memory;
use colors::color;
use constants::*;
use registers::*;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Graphics mode enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphicMode {
    StandardChar,
    MulticolorChar,
    StandardBitmap,
    MulticolorBitmap,
    ExtendedColor,
    Invalid,
}

pub struct Vic {
    registers: [u8; 64],
    raster_counter: u16,
    next_raster_at: u64,
    graphic_mode: GraphicMode,
    border_color: u8,
    background_color: [u8; 4],
    screen_mem_addr: u16,
    char_mem_addr: u16,
    raster_irq_line: u16,
    memory: Weak<RefCell<Memory>>,
    cpu: Weak<RefCell<Cpu>>,
    io: Rc<RefCell<IO>>,
    vic_bank: u8,
}

impl Vic {
    pub fn new(
        memory: Weak<RefCell<Memory>>,
        cpu: Weak<RefCell<Cpu>>,
        io: Rc<RefCell<IO>>,
    ) -> Self {
        let mut vic = Vic {
            registers: [0; 64],
            raster_counter: 0,
            next_raster_at: 0,
            graphic_mode: GraphicMode::StandardChar,
            border_color: 14, // Light blue
            background_color: [6, 0, 0, 0], // Blue
            screen_mem_addr: 0x0400,
            char_mem_addr: 0x1000,
            raster_irq_line: 0,
            memory,
            cpu,
            io,
            vic_bank: 3, // CIA2 PRA default = 0xFF, bits 0-1 = %11, selects actual bank 0 ($0000-$3FFF)
        };
        vic.reset();
        vic
    }

    pub fn reset(&mut self) {
        // Clear all registers
        self.registers = [0; 64];

        // Set default register values
        self.registers[REG_CTRL1 as usize] = 0x1B; // Default: DEN on, 25 rows, Y scroll = 3
        self.registers[REG_CTRL2 as usize] = 0xC8; // Default: 40 cols, X scroll = 0
        self.registers[REG_MEMPTR as usize] = 0x14; // Screen at $0400, chars at $1000
        self.registers[REG_BORDER as usize] = 0x0E; // Light blue border
        self.registers[REG_BGCOLOR0 as usize] = 0x06; // Blue background
        self.registers[REG_IRQ_ENABLE as usize] = 0x00; // IRQs disabled

        // Update internal state
        self.border_color = 14;
        self.background_color = [6, 0, 0, 0];
        self.raster_counter = 0;
        self.next_raster_at = 0;
        self.graphic_mode = GraphicMode::StandardChar;

        self.update_memory_pointers();
        self.update_graphics_mode();
    }

    /// Set VIC bank (called from CIA2 when PRA changes)
    #[allow(dead_code)] // TODO: Wire up CIA2.vic_base_address() to call this
    pub fn set_vic_bank(&mut self, bank: u8) {
        self.vic_bank = bank & 0x03;
    }

    /// Get current VIC bank
    #[allow(dead_code)]
    pub fn get_vic_bank(&self) -> u8 {
        self.vic_bank
    }

    /// Read a VIC register
    pub fn read_register(&self, reg: u8) -> u8 {
        let reg = reg & 0x3F; // Mask to 64 registers

        match reg {
            REG_CTRL1 => {
                // Bit 7 is the 9th bit of the raster counter
                let raster_bit8 = if self.raster_counter > 255 { 0x80 } else { 0x00 };
                (self.registers[reg as usize] & 0x7F) | raster_bit8
            }
            REG_RASTER => {
                // Return low 8 bits of raster counter
                (self.raster_counter & 0xFF) as u8
            }
            REG_IRQ_STATUS => {
                // Unused bits read as 1
                self.registers[reg as usize] | 0x70
            }
            _ => self.registers[reg as usize],
        }
    }

    /// Write to a VIC register
    pub fn write_register(&mut self, reg: u8, value: u8) {
        let reg = reg & 0x3F;

        match reg {
            REG_CTRL1 => {
                self.registers[reg as usize] = value;
                // Update raster IRQ line (bit 8 from CTRL1)
                self.raster_irq_line = (self.raster_irq_line & 0x00FF)
                    | (if value & CTRL1_RST8 != 0 { 0x100 } else { 0 });
                self.update_graphics_mode();
            }
            REG_RASTER => {
                // Writing sets the raster compare value (low 8 bits)
                self.raster_irq_line = (self.raster_irq_line & 0x100) | (value as u16);
            }
            REG_CTRL2 => {
                self.registers[reg as usize] = value;
                self.update_graphics_mode();
            }
            REG_MEMPTR => {
                self.registers[reg as usize] = value;
                self.update_memory_pointers();
            }
            REG_IRQ_STATUS => {
                // Writing 1s acknowledges (clears) the corresponding IRQ bits
                self.acknowledge_irq(value);
            }
            REG_IRQ_ENABLE => {
                self.registers[reg as usize] = value & 0x0F;
            }
            REG_BORDER => {
                self.registers[reg as usize] = value & 0x0F;
                self.border_color = value & 0x0F;
            }
            REG_BGCOLOR0..=REG_BGCOLOR3 => {
                self.registers[reg as usize] = value & 0x0F;
                let idx = (reg - REG_BGCOLOR0) as usize;
                self.background_color[idx] = value & 0x0F;
            }
            _ => {
                self.registers[reg as usize] = value;
            }
        }
    }

    /// Update memory pointers from $D018
    fn update_memory_pointers(&mut self) {
        let memptr = self.registers[REG_MEMPTR as usize];

        // Screen memory: bits 4-7 * 0x400
        self.screen_mem_addr = ((memptr & MEMPTR_SCREEN_MASK) as u16) << 6;

        // Character memory: bits 1-3 * 0x800
        self.char_mem_addr = ((memptr & MEMPTR_CHAR_MASK) as u16) << 10;
    }

    /// Update graphics mode from control registers
    fn update_graphics_mode(&mut self) {
        let ctrl1 = self.registers[REG_CTRL1 as usize];
        let ctrl2 = self.registers[REG_CTRL2 as usize];

        let ecm = ctrl1 & CTRL1_ECM != 0;
        let bmm = ctrl1 & CTRL1_BMM != 0;
        let mcm = ctrl2 & CTRL2_MCM != 0;

        self.graphic_mode = match (ecm, bmm, mcm) {
            (false, false, false) => GraphicMode::StandardChar,
            (false, false, true) => GraphicMode::MulticolorChar,
            (false, true, false) => GraphicMode::StandardBitmap,
            (false, true, true) => GraphicMode::MulticolorBitmap,
            (true, false, false) => GraphicMode::ExtendedColor,
            _ => GraphicMode::Invalid,
        };
    }

    /// Set raster counter and update registers
    fn set_raster_counter(&mut self, value: u16) {
        self.raster_counter = value;
        // Sync with registers (for reads)
        self.registers[REG_RASTER as usize] = (value & 0xFF) as u8;
        // Bit 7 of CTRL1 is synced in read_register
    }

    /// Check if current line is in visible range
    fn is_visible_line(&self) -> bool {
        self.raster_counter >= FIRST_VISIBLE_LINE && self.raster_counter < LAST_VISIBLE_LINE
    }

    /// Check if current line is in display area (where content appears)
    fn is_display_line(&self) -> bool {
        self.raster_counter >= FIRST_DISPLAY_LINE && self.raster_counter < LAST_DISPLAY_LINE
    }

    /// Check if current line is a bad line (VIC steals cycles)
    fn is_bad_line(&self) -> bool {
        if !self.is_display_line() {
            return false;
        }
        let yscroll = self.registers[REG_CTRL1 as usize] & CTRL1_YSCROLL_MASK;
        let den = self.registers[REG_CTRL1 as usize] & CTRL1_DEN != 0;
        den && ((self.raster_counter as u8) & 0x07) == yscroll
    }

    /// Get cycles for current line
    fn cycles_for_line(&self) -> u64 {
        if self.is_bad_line() {
            BAD_LINE_CYCLES
        } else {
            CYCLES_PER_LINE
        }
    }

    /// Draw border for current raster line
    fn draw_border(&mut self, line: u16) {
        let y = (line - FIRST_VISIBLE_LINE) as u32;
        let border_col = color(self.border_color);

        // For lines in the border area (above/below display), fill entire line
        if line < FIRST_DISPLAY_LINE || line >= LAST_DISPLAY_LINE {
            let mut io = self.io.borrow_mut();
            io.fill_rect(0, y, VISIBLE_WIDTH, 1, border_col);
        } else {
            // For display lines, just draw left and right borders
            let mut io = self.io.borrow_mut();
            // Left border
            io.fill_rect(0, y, DISPLAY_START_X, 1, border_col);
            // Right border
            io.fill_rect(
                DISPLAY_START_X + DISPLAY_WIDTH,
                y,
                VISIBLE_WIDTH - DISPLAY_START_X - DISPLAY_WIDTH,
                1,
                border_col,
            );
        }
    }

    /// Draw a line in standard character mode
    fn draw_char_mode_line(&mut self) {
        let Some(memory) = self.memory.upgrade() else {
            return;
        };

        let line = self.raster_counter;
        let y = (line - FIRST_VISIBLE_LINE) as u32;

        // Calculate which character row we're on
        let display_y = (line - FIRST_DISPLAY_LINE) as u32;
        let char_row = display_y / CHAR_HEIGHT;
        let char_line = display_y % CHAR_HEIGHT;

        // Debug: print screen memory after some time
        static mut DEBUG_COUNTER: u32 = 0;
        unsafe {
            DEBUG_COUNTER += 1;
            // Print after ~1 second (50 fps * 1 sec * 200 lines = 10000)
            if DEBUG_COUNTER == 5000 && char_row == 0 && char_line == 0 {
                let mem = memory.borrow();
                print!("Screen mem at ${:04X}: ", self.screen_mem_addr);
                for i in 0..40 {
                    let code = mem.vic_read_byte(self.screen_mem_addr + i, self.vic_bank);
                    print!("{:02X} ", code);
                }
                println!();
                // Also check CPU address directly
                print!("Direct RAM $0400: ");
                for i in 0..40u16 {
                    let code = mem.read_byte(0x0400 + i);
                    print!("{:02X} ", code);
                }
                println!();
            }
        }

        if char_row >= CHAR_ROWS {
            return;
        }

        let mem = memory.borrow();
        let bg_color = color(self.background_color[0]);

        // Draw background first
        self.io
            .borrow_mut()
            .fill_rect(DISPLAY_START_X, y, DISPLAY_WIDTH, 1, bg_color);

        // Draw each character in the row
        for col in 0..CHAR_COLS {
            let screen_offset = char_row * CHAR_COLS + col;

            // Read character code from screen memory
            let char_code = mem.vic_read_byte(
                self.screen_mem_addr + screen_offset as u16,
                self.vic_bank,
            );

            // Read character bitmap line from character memory
            let char_bitmap = mem.vic_read_byte(
                self.char_mem_addr + (char_code as u16) * 8 + char_line as u16,
                self.vic_bank,
            );

            // Read color from color RAM ($D800)
            let char_color = mem.read_byte(0xD800 + screen_offset as u16) & 0x0F;
            let fg_color = color(char_color);

            // Draw 8 pixels
            let x_base = DISPLAY_START_X + col * CHAR_WIDTH;
            let mut io = self.io.borrow_mut();

            for bit in 0..8 {
                let pixel_on = (char_bitmap & (0x80 >> bit)) != 0;
                let pixel_color = if pixel_on { fg_color } else { bg_color };
                io.put_pixel(x_base + bit, y, pixel_color);
            }
        }
    }

    /// Check and trigger raster IRQ
    fn check_raster_irq(&mut self) {
        if self.raster_counter == self.raster_irq_line {
            // Set raster IRQ flag
            self.registers[REG_IRQ_STATUS as usize] |= IRQ_RASTER;

            // Check if raster IRQ is enabled
            if self.registers[REG_IRQ_ENABLE as usize] & IRQ_RASTER != 0 {
                // Set the "any IRQ" flag
                self.registers[REG_IRQ_STATUS as usize] |= IRQ_ANY;

                // Trigger CPU IRQ
                if let Some(cpu) = self.cpu.upgrade() {
                    cpu.borrow_mut().irq();
                }
            }
        }
    }

    /// Acknowledge IRQ by clearing bits that are set in value
    fn acknowledge_irq(&mut self, value: u8) {
        self.registers[REG_IRQ_STATUS as usize] &= !(value & 0x0F);

        // Clear the "any IRQ" flag if no IRQs are pending
        if self.registers[REG_IRQ_STATUS as usize] & 0x0F == 0 {
            self.registers[REG_IRQ_STATUS as usize] &= !IRQ_ANY;
        }
    }

    /// Main emulation step - call this from the main loop
    pub fn step(&mut self) -> bool {
        let Some(cpu) = self.cpu.upgrade() else {
            return false;
        };

        let cpu_cycles = cpu.borrow().cycles() as u64;


        // Process raster lines until we catch up with CPU
        while cpu_cycles >= self.next_raster_at {
            // Draw visible content
            if self.is_visible_line() {
                self.draw_border(self.raster_counter);
                if self.is_display_line() {
                    let den = self.registers[REG_CTRL1 as usize] & CTRL1_DEN != 0;
                    if den {
                        match self.graphic_mode {
                            GraphicMode::StandardChar => self.draw_char_mode_line(),
                            // Other modes not implemented yet
                            _ => {}
                        }
                    }
                }
            }

            // Check for raster interrupt
            self.check_raster_irq();

            // Advance to next raster line
            let new_raster = (self.raster_counter + 1) % SCREEN_LINES;
            self.set_raster_counter(new_raster);

            // On frame complete, refresh display
            if self.raster_counter == 0 {
                self.io.borrow_mut().refresh();
            }

            // Schedule next raster
            self.next_raster_at += self.cycles_for_line();
        }

        true
    }
}
