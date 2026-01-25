pub mod colors;
pub mod constants;
pub mod registers;
pub mod sprites;

use crate::cpu::Cpu;
use crate::io::IO;
use crate::memory::Memory;
use colors::color;
use constants::*;
use registers::*;
use sprites::SpriteState;
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
    bitmap_mem_addr: u16,
    raster_irq_line: u16,
    memory: Weak<RefCell<Memory>>,
    cpu: Weak<RefCell<Cpu>>,
    io: Rc<RefCell<IO>>,
    vic_bank: u8,
    sprites: SpriteState,
    /// Line buffer for sprite compositing (stores pixel colors + transparency info)
    line_buffer: [Option<u32>; 504], // Max visible width with some margin
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
            bitmap_mem_addr: 0x0000,
            raster_irq_line: 0,
            memory,
            cpu,
            io,
            vic_bank: 3, // CIA2 PRA default = 0xFF, bits 0-1 = %11, selects actual bank 0 ($0000-$3FFF)
            sprites: SpriteState::new(),
            line_buffer: [None; 504],
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
        self.bitmap_mem_addr = 0x0000;

        // Reset sprites
        self.sprites.reset();

        // Clear line buffer
        self.line_buffer = [None; 504];

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
            // Sprite X/Y position registers
            0x00..=0x0F => {
                let sprite_num = (reg / 2) as usize;
                if reg & 1 == 0 {
                    // X position (low 8 bits)
                    (self.sprites.x[sprite_num] & 0xFF) as u8
                } else {
                    // Y position
                    self.sprites.y[sprite_num]
                }
            }
            REG_SPRITES_X_MSB => self.sprites.get_x_msb(),
            REG_CTRL1 => {
                // Bit 7 is the 9th bit of the raster counter
                let raster_bit8 = if self.raster_counter > 255 { 0x80 } else { 0x00 };
                (self.registers[reg as usize] & 0x7F) | raster_bit8
            }
            REG_RASTER => {
                // Return low 8 bits of raster counter
                (self.raster_counter & 0xFF) as u8
            }
            REG_LIGHT_PEN_X | REG_LIGHT_PEN_Y => {
                // Light pen registers - return 0 (not implemented)
                0
            }
            REG_SPRITE_ENABLE => self.sprites.enabled,
            REG_SPRITE_Y_EXPAND => self.sprites.expand_y,
            REG_IRQ_STATUS => {
                // Unused bits read as 1
                self.registers[reg as usize] | 0x70
            }
            REG_SPRITE_PRIORITY => self.sprites.priority,
            REG_SPRITE_MULTICOLOR => self.sprites.multicolor,
            REG_SPRITE_X_EXPAND => self.sprites.expand_x,
            REG_SPRITE_COLLISION => {
                // Sprite-sprite collision - clear on read (uses Cell for interior mutability)
                self.sprites.read_collision_sprite()
            }
            REG_SPRITE_BG_COLLISION => {
                // Sprite-background collision - clear on read (uses Cell for interior mutability)
                self.sprites.read_collision_bg()
            }
            REG_SPRITE_MULTICOLOR0 => self.sprites.multicolor0,
            REG_SPRITE_MULTICOLOR1 => self.sprites.multicolor1,
            0x27..=0x2E => {
                // Sprite colors
                let sprite_num = (reg - REG_SPRITE0_COLOR) as usize;
                self.sprites.color[sprite_num]
            }
            _ => self.registers[reg as usize],
        }
    }

    /// Write to a VIC register
    pub fn write_register(&mut self, reg: u8, value: u8) {
        let reg = reg & 0x3F;

        match reg {
            // Sprite X/Y position registers ($D000-$D00F)
            0x00..=0x0F => {
                let sprite_num = (reg / 2) as usize;
                if reg & 1 == 0 {
                    // X position (low 8 bits) - preserve bit 8
                    self.sprites.x[sprite_num] = (self.sprites.x[sprite_num] & 0x100) | (value as u16);
                } else {
                    // Y position
                    self.sprites.y[sprite_num] = value;
                }
                self.registers[reg as usize] = value;
            }
            REG_SPRITES_X_MSB => {
                self.sprites.update_x_msb(value);
                self.registers[reg as usize] = value;
            }
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
            REG_SPRITE_ENABLE => {
                self.sprites.enabled = value;
                self.registers[reg as usize] = value;
            }
            REG_CTRL2 => {
                self.registers[reg as usize] = value;
                self.update_graphics_mode();
            }
            REG_SPRITE_Y_EXPAND => {
                self.sprites.expand_y = value;
                self.registers[reg as usize] = value;
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
            REG_SPRITE_PRIORITY => {
                self.sprites.priority = value;
                self.registers[reg as usize] = value;
            }
            REG_SPRITE_MULTICOLOR => {
                self.sprites.multicolor = value;
                self.registers[reg as usize] = value;
            }
            REG_SPRITE_X_EXPAND => {
                self.sprites.expand_x = value;
                self.registers[reg as usize] = value;
            }
            REG_SPRITE_COLLISION => {
                // Collision registers are read-only, writes are ignored
            }
            REG_SPRITE_BG_COLLISION => {
                // Collision registers are read-only, writes are ignored
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
            REG_SPRITE_MULTICOLOR0 => {
                self.sprites.multicolor0 = value & 0x0F;
                self.registers[reg as usize] = value & 0x0F;
            }
            REG_SPRITE_MULTICOLOR1 => {
                self.sprites.multicolor1 = value & 0x0F;
                self.registers[reg as usize] = value & 0x0F;
            }
            0x27..=0x2E => {
                // Sprite colors
                let sprite_num = (reg - REG_SPRITE0_COLOR) as usize;
                self.sprites.color[sprite_num] = value & 0x0F;
                self.registers[reg as usize] = value & 0x0F;
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

        // Bitmap memory: bit 3 selects $0000 or $2000
        self.bitmap_mem_addr = if memptr & 0x08 != 0 { 0x2000 } else { 0x0000 };
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

    /// Draw a line in character mode (standard or multicolor)
    fn draw_char_mode_line(&mut self, multicolor_mode: bool) {
        let Some(memory) = self.memory.upgrade() else {
            return;
        };

        let line = self.raster_counter;
        let y = (line - FIRST_VISIBLE_LINE) as u32;
        let display_y = (line - FIRST_DISPLAY_LINE) as u32;

        // In 24-row mode, blank the top and bottom 4 lines
        if self.is_24_row_mode() {
            if display_y < 4 || display_y >= 196 {
                let bg_color = color(self.background_color[0]);
                self.io.borrow_mut().fill_rect(DISPLAY_START_X, y, DISPLAY_WIDTH, 1, bg_color);
                return;
            }
        }

        // Calculate which character row and line within the character we're on
        let char_row = display_y / CHAR_HEIGHT;
        let char_line = display_y % CHAR_HEIGHT;

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
            let char_color = mem.read_byte(0xD800 + screen_offset as u16);

            // Calculate base X position
            let x_base = DISPLAY_START_X + col * CHAR_WIDTH;

            // Determine if this character uses multicolor mode
            // In multicolor char mode, bit 3 of color RAM determines the mode per character
            let use_multicolor = multicolor_mode && (char_color & 0x08) != 0;

            let mut io = self.io.borrow_mut();

            if use_multicolor {
                // Multicolor mode: 2 bits per pixel, 4 pixels wide (displayed as 8 pixels)
                // Colors: 00=bg0, 01=bg1, 10=bg2, 11=fg (lower 3 bits of color RAM)
                let fg_color = color(char_color & 0x07);
                let bg1_color = color(self.background_color[1]);
                let bg2_color = color(self.background_color[2]);

                for pair in 0..4 {
                    let shift = 6 - (pair * 2);
                    let bits = (char_bitmap >> shift) & 0x03;
                    let pixel_color = match bits {
                        0 => bg_color,
                        1 => bg1_color,
                        2 => bg2_color,
                        3 => fg_color,
                        _ => unreachable!(),
                    };

                    // Each MC pixel is 2 screen pixels wide
                    let x_pos = x_base + (pair as u32) * 2;
                    io.put_pixel(x_pos, y, pixel_color);
                    io.put_pixel(x_pos + 1, y, pixel_color);
                }
            } else {
                // Standard hires mode: 1 bit per pixel
                let fg_color = color(char_color & 0x0F);

                for bit in 0..8 {
                    let pixel_on = (char_bitmap & (0x80 >> bit)) != 0;
                    let pixel_color = if pixel_on { fg_color } else { bg_color };
                    io.put_pixel(x_base + bit, y, pixel_color);
                }
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

    // =====================================================
    // Hardware Scrolling Helpers
    // =====================================================

    /// Get X scroll value (0-7 pixels) from $D016 bits 0-2
    #[inline]
    #[allow(dead_code)]
    fn x_scroll(&self) -> u8 {
        self.registers[REG_CTRL2 as usize] & CTRL2_XSCROLL_MASK
    }

    /// Get Y scroll value (0-7 pixels) from $D011 bits 0-2
    #[inline]
    #[allow(dead_code)]
    fn y_scroll(&self) -> u8 {
        self.registers[REG_CTRL1 as usize] & CTRL1_YSCROLL_MASK
    }

    /// Check if 38-column mode is enabled ($D016 bit 3 = 0)
    #[inline]
    #[allow(dead_code)]
    fn is_38_column_mode(&self) -> bool {
        (self.registers[REG_CTRL2 as usize] & CTRL2_CSEL) == 0
    }

    /// Check if 24-row mode is enabled ($D011 bit 3 = 0)
    #[inline]
    fn is_24_row_mode(&self) -> bool {
        (self.registers[REG_CTRL1 as usize] & CTRL1_RSEL) == 0
    }

    // =====================================================
    // Bitmap Mode Rendering
    // =====================================================

    /// Draw a line in standard bitmap mode (320x200 hires)
    fn draw_bitmap_mode_line(&mut self) {
        let Some(memory) = self.memory.upgrade() else {
            return;
        };

        let line = self.raster_counter;
        let y = (line - FIRST_VISIBLE_LINE) as u32;
        let display_y = (line - FIRST_DISPLAY_LINE) as u32;

        // Handle 24-row mode
        if self.is_24_row_mode() && (display_y < 4 || display_y >= 196) {
            let bg_color = color(self.background_color[0]);
            self.io.borrow_mut().fill_rect(DISPLAY_START_X, y, DISPLAY_WIDTH, 1, bg_color);
            return;
        }

        let mem = memory.borrow();

        // Calculate character row and line within character
        let char_row = display_y / CHAR_HEIGHT;
        let char_line = display_y % CHAR_HEIGHT;

        if char_row >= CHAR_ROWS {
            return;
        }

        // Draw each 8-pixel cell
        for col in 0..CHAR_COLS {
            // Bitmap address: base + (char_row * 40 + col) * 8 + char_line
            let bitmap_offset = (char_row * CHAR_COLS + col) * 8 + char_line;
            let bitmap_byte = mem.vic_read_byte(
                self.bitmap_mem_addr + bitmap_offset as u16,
                self.vic_bank,
            );

            // Color from screen memory (same layout as char mode)
            // High nibble = foreground, low nibble = background
            let screen_offset = char_row * CHAR_COLS + col;
            let color_byte = mem.vic_read_byte(
                self.screen_mem_addr + screen_offset as u16,
                self.vic_bank,
            );

            let fg_color = color((color_byte >> 4) & 0x0F);
            let bg_color = color(color_byte & 0x0F);

            let x_base = DISPLAY_START_X + col * CHAR_WIDTH;
            let mut io = self.io.borrow_mut();

            // Draw 8 pixels
            for bit in 0..8 {
                let pixel_on = (bitmap_byte & (0x80 >> bit)) != 0;
                let pixel_color = if pixel_on { fg_color } else { bg_color };
                io.put_pixel(x_base + bit, y, pixel_color);
            }
        }
    }

    /// Draw a line in multicolor bitmap mode
    fn draw_mc_bitmap_mode_line(&mut self) {
        let Some(memory) = self.memory.upgrade() else {
            return;
        };

        let line = self.raster_counter;
        let y = (line - FIRST_VISIBLE_LINE) as u32;
        let display_y = (line - FIRST_DISPLAY_LINE) as u32;

        // Handle 24-row mode
        if self.is_24_row_mode() && (display_y < 4 || display_y >= 196) {
            let bg_color = color(self.background_color[0]);
            self.io.borrow_mut().fill_rect(DISPLAY_START_X, y, DISPLAY_WIDTH, 1, bg_color);
            return;
        }

        let mem = memory.borrow();

        let char_row = display_y / CHAR_HEIGHT;
        let char_line = display_y % CHAR_HEIGHT;

        if char_row >= CHAR_ROWS {
            return;
        }

        let bg0_color = color(self.background_color[0]);

        // Draw each 8-pixel cell (4 multicolor pixels)
        for col in 0..CHAR_COLS {
            // Bitmap address
            let bitmap_offset = (char_row * CHAR_COLS + col) * 8 + char_line;
            let bitmap_byte = mem.vic_read_byte(
                self.bitmap_mem_addr + bitmap_offset as u16,
                self.vic_bank,
            );

            // Colors from screen memory and color RAM
            let screen_offset = char_row * CHAR_COLS + col;
            let screen_byte = mem.vic_read_byte(
                self.screen_mem_addr + screen_offset as u16,
                self.vic_bank,
            );
            let color_ram_byte = mem.read_byte(0xD800 + screen_offset as u16);

            // MC bitmap colors:
            // 00 = $D021 (background 0)
            // 01 = screen memory high nibble
            // 10 = screen memory low nibble
            // 11 = color RAM
            let color1 = color((screen_byte >> 4) & 0x0F);
            let color2 = color(screen_byte & 0x0F);
            let color3 = color(color_ram_byte & 0x0F);

            let x_base = DISPLAY_START_X + col * CHAR_WIDTH;
            let mut io = self.io.borrow_mut();

            // Draw 4 multicolor pixels (each 2 screen pixels wide)
            for pair in 0..4 {
                let shift = 6 - (pair * 2);
                let bits = (bitmap_byte >> shift) & 0x03;
                let pixel_color = match bits {
                    0 => bg0_color,
                    1 => color1,
                    2 => color2,
                    3 => color3,
                    _ => unreachable!(),
                };

                let x_pos = x_base + (pair as u32) * 2;
                io.put_pixel(x_pos, y, pixel_color);
                io.put_pixel(x_pos + 1, y, pixel_color);
            }
        }
    }

    /// Draw a line in extended color mode
    fn draw_extended_color_mode_line(&mut self) {
        let Some(memory) = self.memory.upgrade() else {
            return;
        };

        let line = self.raster_counter;
        let y = (line - FIRST_VISIBLE_LINE) as u32;
        let display_y = (line - FIRST_DISPLAY_LINE) as u32;

        if self.is_24_row_mode() && (display_y < 4 || display_y >= 196) {
            let bg_color = color(self.background_color[0]);
            self.io.borrow_mut().fill_rect(DISPLAY_START_X, y, DISPLAY_WIDTH, 1, bg_color);
            return;
        }

        let char_row = display_y / CHAR_HEIGHT;
        let char_line = display_y % CHAR_HEIGHT;

        if char_row >= CHAR_ROWS {
            return;
        }

        let mem = memory.borrow();

        // Draw background first
        let default_bg = color(self.background_color[0]);
        self.io.borrow_mut().fill_rect(DISPLAY_START_X, y, DISPLAY_WIDTH, 1, default_bg);

        for col in 0..CHAR_COLS {
            let screen_offset = char_row * CHAR_COLS + col;
            let char_code = mem.vic_read_byte(
                self.screen_mem_addr + screen_offset as u16,
                self.vic_bank,
            );

            // In ECM, top 2 bits of char code select background color
            let bg_select = (char_code >> 6) & 0x03;
            let actual_char = char_code & 0x3F; // Only 64 characters available

            let bg_color = color(self.background_color[bg_select as usize]);

            let char_bitmap = mem.vic_read_byte(
                self.char_mem_addr + (actual_char as u16) * 8 + char_line as u16,
                self.vic_bank,
            );

            let char_color = mem.read_byte(0xD800 + screen_offset as u16) & 0x0F;
            let fg_color = color(char_color);

            let x_base = DISPLAY_START_X + col * CHAR_WIDTH;
            let mut io = self.io.borrow_mut();

            for bit in 0..8 {
                let pixel_on = (char_bitmap & (0x80 >> bit)) != 0;
                let pixel_color = if pixel_on { fg_color } else { bg_color };
                io.put_pixel(x_base + bit, y, pixel_color);
            }
        }
    }

    // =====================================================
    // Sprite Rendering
    // =====================================================

    /// Get sprite pointer for a given sprite number
    fn get_sprite_pointer(&self, sprite_num: usize) -> u16 {
        let Some(memory) = self.memory.upgrade() else {
            return 0;
        };
        let mem = memory.borrow();

        // Sprite pointers are at screen_mem + $3F8 + sprite_num
        let pointer_addr = self.screen_mem_addr + SPRITE_POINTER_OFFSET + sprite_num as u16;
        let pointer = mem.vic_read_byte(pointer_addr, self.vic_bank);

        // Sprite data address = pointer * 64
        pointer as u16 * 64
    }

    /// Draw all sprites for the current raster line
    #[allow(dead_code)]
    fn draw_sprites(&mut self) {
        let line = self.raster_counter;

        // Skip if no sprites are enabled
        if self.sprites.enabled == 0 {
            return;
        }

        let Some(memory) = self.memory.upgrade() else {
            return;
        };

        let screen_y = (line - FIRST_VISIBLE_LINE) as i32;

        // Process sprites in reverse order (7 to 0) so lower numbered sprites appear on top
        for sprite_num in (0..NUM_SPRITES).rev() {
            if !self.sprites.is_enabled(sprite_num) {
                continue;
            }

            let sprite_y = self.sprites.y[sprite_num] as i32;
            let sprite_x = self.sprites.x[sprite_num] as i32;

            // Sprite Y position is relative to the top of the visible area
            // Sprites appear at Y position + 50 (first visible line offset)
            let sprite_screen_y = sprite_y - 50 + FIRST_VISIBLE_LINE as i32;

            let height = self.sprites.get_height(sprite_num) as i32;

            // Check if this sprite is visible on this line
            if screen_y < sprite_screen_y - FIRST_VISIBLE_LINE as i32
                || screen_y >= sprite_screen_y - FIRST_VISIBLE_LINE as i32 + height
            {
                continue;
            }

            // Calculate which line of the sprite we're drawing
            let sprite_line = if self.sprites.is_expanded_y(sprite_num) {
                ((screen_y - (sprite_screen_y - FIRST_VISIBLE_LINE as i32)) / 2) as u32
            } else {
                (screen_y - (sprite_screen_y - FIRST_VISIBLE_LINE as i32)) as u32
            };

            if sprite_line >= SPRITE_HEIGHT {
                continue;
            }

            // Get sprite data address
            let sprite_data_addr = self.get_sprite_pointer(sprite_num);

            let mem = memory.borrow();

            // Read the 3 bytes for this sprite line
            let line_offset = sprite_line * SPRITE_BYTES_PER_LINE;
            let byte0 = mem.vic_read_byte(sprite_data_addr + line_offset as u16, self.vic_bank);
            let byte1 = mem.vic_read_byte(sprite_data_addr + line_offset as u16 + 1, self.vic_bank);
            let byte2 = mem.vic_read_byte(sprite_data_addr + line_offset as u16 + 2, self.vic_bank);

            let sprite_data = [byte0, byte1, byte2];

            // Drop the borrow before calling render methods
            drop(mem);

            // Render the sprite line
            if self.sprites.is_multicolor(sprite_num) {
                self.draw_mc_sprite_line(sprite_num, sprite_x, screen_y as u32, &sprite_data);
            } else {
                self.draw_hires_sprite_line(sprite_num, sprite_x, screen_y as u32, &sprite_data);
            }
        }
    }

    /// Draw a single line of a hires (single color) sprite
    fn draw_hires_sprite_line(&mut self, sprite_num: usize, sprite_x: i32, screen_y: u32, data: &[u8; 3]) {
        let sprite_color = color(self.sprites.color[sprite_num]);
        let expanded_x = self.sprites.is_expanded_x(sprite_num);
        let width_multiplier = if expanded_x { 2 } else { 1 };

        let mut io = self.io.borrow_mut();

        // Process 24 pixels (3 bytes * 8 bits)
        for byte_idx in 0..3 {
            let byte = data[byte_idx];
            for bit in 0..8 {
                if byte & (0x80 >> bit) != 0 {
                    // Pixel is set
                    let pixel_x = sprite_x + ((byte_idx * 8 + bit) as i32) * width_multiplier;

                    // Draw pixel(s) if within visible area
                    // Sprite X is in VIC coordinates (24 = left edge of display)
                    let screen_x = pixel_x - 24 + DISPLAY_START_X as i32;

                    if screen_x >= 0 && (screen_x as u32) < VISIBLE_WIDTH {
                        io.put_pixel(screen_x as u32, screen_y, sprite_color);
                        if expanded_x && screen_x + 1 >= 0 && ((screen_x + 1) as u32) < VISIBLE_WIDTH {
                            io.put_pixel((screen_x + 1) as u32, screen_y, sprite_color);
                        }
                    }
                }
            }
        }
    }

    /// Draw a single line of a multicolor sprite
    fn draw_mc_sprite_line(&mut self, sprite_num: usize, sprite_x: i32, screen_y: u32, data: &[u8; 3]) {
        let sprite_color = color(self.sprites.color[sprite_num]);
        let mc0_color = color(self.sprites.multicolor0);
        let mc1_color = color(self.sprites.multicolor1);
        let expanded_x = self.sprites.is_expanded_x(sprite_num);
        let width_multiplier = if expanded_x { 2 } else { 1 };

        let mut io = self.io.borrow_mut();

        // Process 12 multicolor pixels (3 bytes * 4 pixel pairs)
        for byte_idx in 0..3 {
            let byte = data[byte_idx];
            for pair in 0..4 {
                let shift = 6 - (pair * 2);
                let bits = (byte >> shift) & 0x03;

                // MC sprite colors: 00=transparent, 01=multicolor0, 10=sprite color, 11=multicolor1
                let pixel_color = match bits {
                    0 => None, // Transparent
                    1 => Some(mc0_color),
                    2 => Some(sprite_color),
                    3 => Some(mc1_color),
                    _ => unreachable!(),
                };

                if let Some(color) = pixel_color {
                    // Each MC pixel is 2 hires pixels wide
                    let base_pixel_x = sprite_x + ((byte_idx * 4 + pair) as i32) * 2 * width_multiplier;
                    let screen_x = base_pixel_x - 24 + DISPLAY_START_X as i32;

                    // Draw 2 pixels (or 4 if X-expanded)
                    let pixel_width = 2 * width_multiplier;
                    for px in 0..pixel_width {
                        let x = screen_x + px;
                        if x >= 0 && (x as u32) < VISIBLE_WIDTH {
                            io.put_pixel(x as u32, screen_y, color);
                        }
                    }
                }
            }
        }
    }

    /// Draw sprites that appear behind the background (priority = 1)
    fn draw_sprites_behind(&mut self) {
        // Only draw sprites with priority bit set (appear behind background)
        for sprite_num in (0..NUM_SPRITES).rev() {
            if self.sprites.is_enabled(sprite_num) && self.sprites.has_priority_behind(sprite_num) {
                self.draw_single_sprite(sprite_num);
            }
        }
    }

    /// Draw sprites that appear in front of the background (priority = 0)
    fn draw_sprites_front(&mut self) {
        // Only draw sprites with priority bit clear (appear in front of background)
        for sprite_num in (0..NUM_SPRITES).rev() {
            if self.sprites.is_enabled(sprite_num) && !self.sprites.has_priority_behind(sprite_num) {
                self.draw_single_sprite(sprite_num);
            }
        }
    }

    /// Draw a single sprite (called from draw_sprites_behind/front)
    fn draw_single_sprite(&mut self, sprite_num: usize) {
        let line = self.raster_counter;

        let Some(memory) = self.memory.upgrade() else {
            return;
        };

        let screen_y = (line - FIRST_VISIBLE_LINE) as i32;
        let sprite_y = self.sprites.y[sprite_num] as i32;
        let sprite_x = self.sprites.x[sprite_num] as i32;

        let sprite_screen_y = sprite_y - 50 + FIRST_VISIBLE_LINE as i32;
        let height = self.sprites.get_height(sprite_num) as i32;

        if screen_y < sprite_screen_y - FIRST_VISIBLE_LINE as i32
            || screen_y >= sprite_screen_y - FIRST_VISIBLE_LINE as i32 + height
        {
            return;
        }

        let sprite_line = if self.sprites.is_expanded_y(sprite_num) {
            ((screen_y - (sprite_screen_y - FIRST_VISIBLE_LINE as i32)) / 2) as u32
        } else {
            (screen_y - (sprite_screen_y - FIRST_VISIBLE_LINE as i32)) as u32
        };

        if sprite_line >= SPRITE_HEIGHT {
            return;
        }

        let sprite_data_addr = self.get_sprite_pointer(sprite_num);
        let mem = memory.borrow();

        let line_offset = sprite_line * SPRITE_BYTES_PER_LINE;
        let byte0 = mem.vic_read_byte(sprite_data_addr + line_offset as u16, self.vic_bank);
        let byte1 = mem.vic_read_byte(sprite_data_addr + line_offset as u16 + 1, self.vic_bank);
        let byte2 = mem.vic_read_byte(sprite_data_addr + line_offset as u16 + 2, self.vic_bank);

        let sprite_data = [byte0, byte1, byte2];
        drop(mem);

        if self.sprites.is_multicolor(sprite_num) {
            self.draw_mc_sprite_line(sprite_num, sprite_x, screen_y as u32, &sprite_data);
        } else {
            self.draw_hires_sprite_line(sprite_num, sprite_x, screen_y as u32, &sprite_data);
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
                        // Draw sprites with priority=1 (behind background) first
                        self.draw_sprites_behind();

                        // Draw background graphics
                        match self.graphic_mode {
                            GraphicMode::StandardChar => self.draw_char_mode_line(false),
                            GraphicMode::MulticolorChar => self.draw_char_mode_line(true),
                            GraphicMode::StandardBitmap => self.draw_bitmap_mode_line(),
                            GraphicMode::MulticolorBitmap => self.draw_mc_bitmap_mode_line(),
                            GraphicMode::ExtendedColor => self.draw_extended_color_mode_line(),
                            GraphicMode::Invalid => {
                                // Invalid mode - just draw black
                                let y = (self.raster_counter - FIRST_VISIBLE_LINE) as u32;
                                self.io.borrow_mut().fill_rect(DISPLAY_START_X, y, DISPLAY_WIDTH, 1, 0xFF000000);
                            }
                        }

                        // Draw sprites with priority=0 (in front of background)
                        self.draw_sprites_front();
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
