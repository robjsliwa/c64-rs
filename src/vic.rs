use super::cpu::Cpu;
use super::io::IO;
use super::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum GraphicsMode {
    CharMode,
    MCCharMode,
    BitmapMode,
    MCBitmapMode,
    ExitBgMode,
    IllegalMode,
}

pub struct Vic<'a> {
    mem: Rc<RefCell<Memory<'a>>>,
    cpu: Rc<RefCell<Cpu<'a>>>,
    io: Rc<RefCell<IO<'a>>>,

    // Arrays of 8-bit unsigned integers
    mx: [u8; 8],
    my: [u8; 8],
    msbx: u8,
    sprite_enabled: u8,
    sprite_priority: u8,
    sprite_multicolor: u8,
    sprite_double_width: u8,
    sprite_double_height: u8,
    sprite_shared_colors: [u8; 2],
    sprite_colors: [u8; 8],

    // Other unsigned integers and arrays
    border_color: u8,
    bgcolor: [u8; 4],

    // Control registers
    cr1: u8,
    cr2: u8,

    // Raster related attributes
    next_raster_at: u32, // assuming unsigned int is mapped to u32
    frame_c: u32,
    raster_c: u8,
    raster_irq: i32, // assuming int is mapped to i32

    // Interrupt control attributes
    irq_status: u8,
    irq_enabled: u8,

    // Screen, character memory, and bitmap addresses
    screen_mem: u16,
    char_mem: u16,
    bitmap_mem: u16,
    mem_pointers: u8,

    // Graphic mode
    graphic_mode: GraphicsMode,
}

impl<'a> Vic<'a> {
    pub const SCREEN_LINES: u32 = 312;
    pub const SCREEN_COLUMNS: u32 = 504;
    pub const VISIBLE_SCREEN_WIDTH: u32 = 403;
    pub const VISIBLE_SCREEN_HEIGHT: u32 = 284;
    pub const FIRST_VISIBLE_LINE: u32 = 14;
    pub const LAST_VISIBLE_LINE: u32 = 298;
    pub const LINE_CYCLES: u32 = 63;
    pub const BAD_LINE_CYCLES: u32 = 23;
    pub const REFRESH_RATE: f64 = 1f64 / 50.125;
    pub const SPRITE_PTRS_OFFSET: u32 = 0x3f8;

    // Graphics constants
    pub const G_RES_X: u32 = 320;
    pub const G_RES_Y: u32 = 200;
    pub const G_COLS: u32 = 40;
    pub const G_ROWS: u32 = 25;
    pub const G_FIRST_LINE: u32 = 56;
    pub const G_LAST_LINE: u32 = 256;
    pub const G_FIRST_COL: u32 = 42;

    // Sprites
    pub const SPRITE_WIDTH: u32 = 24;
    pub const SPRITE_HEIGHT: u32 = 21;
    pub const SPRITE_SIZE: u32 = 64;
    pub const SPRITES_FIRST_LINE: u32 = 6;
    pub const SPRITES_FIRST_COL: u32 = 18;

    pub fn new(
        mem: Rc<RefCell<Memory<'a>>>,
        cpu: Rc<RefCell<Cpu<'a>>>,
        io: Rc<RefCell<IO<'a>>>,
    ) -> Self {
        Vic {
            mem,
            cpu,
            io,

            // Initialize raster related attributes
            raster_irq: 0,
            raster_c: 0,
            irq_enabled: 0,
            irq_status: 0,
            next_raster_at: Vic::LINE_CYCLES,

            // Initialize sprite attributes
            mx: [0; 8],
            my: [0; 8],
            sprite_colors: [0; 8],
            msbx: 0,
            sprite_double_height: 0,
            sprite_double_width: 0,
            sprite_enabled: 0,
            sprite_priority: 0,
            sprite_multicolor: 0,
            sprite_shared_colors: [0, 0],

            // Initialize color attributes
            border_color: 0,
            bgcolor: [0, 0, 0, 0],

            // Initialize control registers
            cr1: 0,
            cr2: 0,

            // Initialize frame counter
            frame_c: 0,

            // Initialize default memory pointers
            screen_mem: Memory::BASE_ADDR_SCREEN,
            char_mem: Memory::BASE_ADDR_CHARS,
            bitmap_mem: Memory::BASE_ADDR_BITMAP,

            // Bit 0 is unused
            mem_pointers: 1 << 0,

            // Current graphic mode
            graphic_mode: GraphicsMode::CharMode,
        }
    }

    pub fn step(&mut self) -> bool {
        // If there are unacknowledged interrupts, raise an interrupt again
        if self.read_register(0x19) & 0x80 != 0 {
            self.cpu.borrow_mut().irq();
        }

        // Are we at the next raster line?
        if self.cpu.borrow().cycles() >= self.next_raster_at {
            let rstr = self.raster_counter();
            // Check raster IRQs
            if self.raster_irq_enabled() && rstr == self.raster_irq {
                // Set interrupt origin (raster)
                self.irq_status |= 1 << 0;
                // Raise interrupt
                self.cpu.borrow_mut().irq();
            }
            if rstr >= Vic::FIRST_VISIBLE_LINE as i32 && rstr < Vic::LAST_VISIBLE_LINE as i32 {
                // Draw border
                let screen_y = rstr - Vic::FIRST_VISIBLE_LINE as i32;
                self.io
                    .borrow_mut()
                    .screen_draw_border(screen_y.try_into().unwrap(), self.border_color.into());
                // Draw raster on current graphic mode
                match self.graphic_mode {
                    GraphicsMode::CharMode | GraphicsMode::MCCharMode => {
                        self.draw_raster_char_mode()
                    }
                    GraphicsMode::BitmapMode | GraphicsMode::MCBitmapMode => {
                        self.draw_raster_bitmap_mode()
                    }
                    _ => {
                        println!("Unsupported graphic mode: {:?}", self.graphic_mode);
                        return false;
                    }
                }
                // Draw sprites
                self.draw_raster_sprites();
            }
            // Next raster
            self.next_raster_at += if self.is_bad_line() {
                Vic::BAD_LINE_CYCLES
            } else {
                Vic::LINE_CYCLES
            };
            // Update raster
            self.raster_counter_set(rstr + 1);
            if rstr >= Vic::SCREEN_LINES as i32 {
                self.io.borrow_mut().screen_refresh();
                self.frame_c += 1;
                self.raster_counter_set(0);
            }
        }
        true
    }

    pub fn read_register(&self, r: u8) -> u8 {
        match r {
            // Get X coord of sprite n
            0x0 | 0x2 | 0x4 | 0x6 | 0x8 | 0xc | 0xe => self.mx[r as usize >> 1],
            // Get Y coord of sprite n
            0x1 | 0x3 | 0x5 | 0x7 | 0x9 | 0xb | 0xd | 0xf => self.my[r as usize >> 1],
            // MSBs of sprites X coordinates
            0x10 => self.msbx,
            // Control register 1
            0x11 => self.cr1,
            // Raster counter
            0x12 => self.raster_c,
            // Sprite enable register
            0x15 => self.sprite_enabled,
            // Control register 2
            0x16 => self.cr2,
            // Sprite double height
            0x17 => self.sprite_double_height,
            // Memory pointers
            0x18 => self.mem_pointers,
            // Interrupt status register
            0x19 => {
                let mut retval = 0xf & self.irq_status;
                if retval != 0 {
                    retval |= 0x80
                } // IRQ bit
                retval |= 0x70; // non-connected bits (always set)
                retval
            }
            // Interrupt enable register
            0x1a => 0xf0 | self.irq_enabled,
            // Sprite priority register
            0x1b => self.sprite_priority,
            // Sprite multicolor mode
            0x1c => self.sprite_multicolor,
            // Sprite double width
            0x1d => self.sprite_double_width,
            // Border color
            0x20 => self.border_color,
            // Background colors
            0x21..=0x24 => self.bgcolor[r as usize - 0x21],
            // Sprite colors
            0x25..=0x26 => self.sprite_shared_colors[r as usize - 0x25],
            0x27..=0x2e => self.sprite_colors[r as usize - 0x27],
            // Unused
            0x2f..=0x3f => 0xff,
            // Default case
            _ => 0xff,
        }
    }

    pub fn write_register(&mut self, r: u8, v: u8) {
        match r {
            // Store X coord of sprite n
            0x0 | 0x2 | 0x4 | 0x6 | 0x8 | 0xc | 0xe => self.mx[r as usize >> 1] = v,
            // Store Y coord of sprite n
            0x1 | 0x3 | 0x5 | 0x7 | 0x9 | 0xb | 0xd | 0xf => self.my[r as usize >> 1] = v,
            // MSBs of X coordinates
            0x10 => self.msbx = v,
            // Control register 1
            0x11 => {
                self.cr1 = v & 0x7f;
                self.raster_irq &= 0xff;
                // TODO: Check this cast
                self.raster_irq |= ((v as u16 & 0x80) << 1) as i32;
                self.set_graphic_mode();
            }
            // Raster irq
            // TODO: Check this cast
            0x12 => self.raster_irq = (v as i32) | self.raster_irq & (1 << 8),
            // Sprite enable register
            0x15 => self.sprite_enabled = v,
            // Control register 2
            0x16 => {
                self.cr2 = v;
                self.set_graphic_mode();
            }
            // Sprite double height
            0x17 => self.sprite_double_height = v,
            // Memory pointers
            0x18 => {
                self.char_mem = ((v as u32 & 0xe) << 10).try_into().unwrap();
                self.screen_mem = ((v & 0xf0) << 6).try_into().unwrap();
                self.bitmap_mem = ((v as u32 & 0x8) << 10).try_into().unwrap();
                self.mem_pointers = v | (1 << 0);
            }
            // Interrupt request register
            0x19 => self.irq_status &= !(v & 0xf),
            // Interrupt enable register
            0x1a => self.irq_enabled = v,
            // Sprite priority register
            0x1b => self.sprite_priority = v,
            // Sprite multicolor mode
            0x1c => self.sprite_multicolor = v,
            // Sprite double width
            0x1d => self.sprite_double_width = v,
            // Border color
            0x20 => self.border_color = v,
            // Background colors
            0x21..=0x24 => self.bgcolor[r as usize - 0x21] = v,
            // Sprite colors
            0x25..=0x26 => self.sprite_shared_colors[r as usize - 0x25] = v,
            0x27..=0x2e => self.sprite_colors[r as usize - 0x27] = v,
            // Unused
            0x2f..=0x3f => (),
            // Default case
            _ => (),
        }
    }

    pub fn set_graphic_mode(&mut self) {
        let ecm = (self.cr1 & (1 << 6)) != 0;
        let bmm = (self.cr1 & (1 << 5)) != 0;
        let mcm = (self.cr2 & (1 << 4)) != 0;

        self.graphic_mode = if !ecm && !bmm && !mcm {
            GraphicsMode::CharMode
        } else if !ecm && !bmm && mcm {
            GraphicsMode::MCCharMode
        } else if !ecm && bmm && !mcm {
            GraphicsMode::BitmapMode
        } else if !ecm && bmm && mcm {
            GraphicsMode::MCBitmapMode
        } else if ecm && !bmm && !mcm {
            GraphicsMode::ExitBgMode
        } else {
            GraphicsMode::IllegalMode
        };
    }

    // Retrieves a character from screen memory
    pub fn get_screen_char(&self, column: u32, row: u32) -> u8 {
        let addr = self.screen_mem as u32 + (row * Vic::G_COLS) + column;
        // TODO: Check this cast
        self.mem.borrow().vic_read_byte(addr.try_into().unwrap())
    }

    // Retrieves color RAM for given screen coordinates
    pub fn get_char_color(&self, column: u32, row: u32) -> u8 {
        let addr = Memory::ADDR_COLOR_RAM as u32 + (row * Vic::G_COLS) + column;
        // TODO: Check this cast
        self.mem.borrow().read_byte_no_io(addr.try_into().unwrap()) & 0x0f
    }

    // Retrieves pixel data from character memory
    pub fn get_char_data(&self, chr: i32, line: i32) -> u8 {
        let addr = self.char_mem as u32 + (chr * 8) as u32 + line as u32;
        // TODO: Check this cast
        self.mem.borrow().vic_read_byte(addr.try_into().unwrap())
    }

    // Retrieves pixel data from bitmap memory
    pub fn get_bitmap_data(&self, column: u32, row: u32, line: u32) -> u8 {
        let addr = self.bitmap_mem as u32 + ((row * Vic::G_COLS + column) * 8 + line);
        // TODO: Check this cast
        self.mem.borrow().vic_read_byte(addr.try_into().unwrap())
    }

    // Gets sprite pointer (n is sprite # 0-7)
    pub fn get_sprite_ptr(&self, n: u32) -> u16 {
        // TODO: Check this cast
        let ptraddr = self.screen_mem as u32 + Vic::SPRITE_PTRS_OFFSET + n;
        Vic::SPRITE_SIZE as u16
            * self.mem.borrow().vic_read_byte(ptraddr.try_into().unwrap()) as u16
    }

    pub fn draw_char(&self, x: u32, y: u32, data: u8, color: u8) {
        for i in 0..8 {
            let xoffs = x + 8 - i + self.horizontal_scroll() as u32;
            // Don't draw outside (due to horizontal scroll)
            if xoffs > Vic::G_FIRST_COL + Vic::G_RES_X {
                continue;
            }
            // Draw pixel if the bit is set
            if data & (1 << i) != 0 {
                self.io
                    .borrow_mut()
                    .screen_update_pixel(xoffs, y, color as u32);
            }
        }
    }

    pub fn draw_mcchar(&self, x: u32, y: u32, data: u8, color: u8) {
        for i in 0..4 {
            // Color source
            let cs = ((data >> (i * 2)) & 0x3) as usize;
            // Determine color
            let c = match cs {
                0 => self.bgcolor[0],
                1 => self.bgcolor[1],
                2 => self.bgcolor[2],
                3 => color,
                _ => unreachable!(), // This case should not happen
            };

            let xoffs = x + 8 - i * 2 + self.horizontal_scroll() as u32;
            // Update pixels
            self.io.borrow_mut().screen_update_pixel(xoffs, y, c.into());
            self.io
                .borrow_mut()
                .screen_update_pixel(xoffs + 1, y, c.into());
        }
    }

    pub fn draw_raster_char_mode(&self) {
        let rstr = self.raster_counter();
        let y = rstr - Vic::FIRST_VISIBLE_LINE as i32;
        if rstr >= Vic::G_FIRST_LINE as i32
            && rstr < Vic::G_LAST_LINE as i32
            && !self.is_screen_off()
        {
            // Draw background
            self.io.borrow_mut().screen_draw_rect(
                Vic::G_FIRST_COL,
                y.try_into().unwrap(),
                Vic::G_RES_X,
                self.bgcolor[0].into(),
            );
            // Draw characters
            for column in 0..Vic::G_COLS {
                // Check 38 cols mode
                if (self.cr2 & (1 << 3)) == 0 && (column == 0 || column == Vic::G_COLS - 1) {
                    continue;
                }
                let x = Vic::G_FIRST_COL + column * 8;
                let line = rstr - Vic::G_FIRST_LINE as i32;
                let row = line / 8;
                let char_row = line % 8;
                // Retrieve screen character
                let c = self.get_screen_char(column, row.try_into().unwrap());
                // Retrieve character bitmap data
                let data = self.get_char_data(c.into(), char_row);
                // Retrieve color data
                let color = self.get_char_color(column, row.try_into().unwrap());
                // Draw character
                if self.graphic_mode == GraphicsMode::MCCharMode && (color & (1 << 3)) != 0 {
                    self.draw_mcchar(x, y.try_into().unwrap(), data, color & 0x7);
                } else {
                    self.draw_char(x, y.try_into().unwrap(), data, color);
                }
            }
        }
    }

    pub fn draw_bitmap(&self, x: u32, y: u32, data: u8, color: u8) {
        let forec = (color >> 4) & 0xf;
        let bgc = color & 0xf;
        for i in 0..8 {
            let xoffs = x + 8 - i + self.horizontal_scroll() as u32;
            // Don't draw outside (due to horizontal scroll)
            if xoffs > Vic::G_FIRST_COL + Vic::G_RES_X {
                continue;
            }
            // Draw pixel
            if data & (1 << i) != 0 {
                self.io
                    .borrow_mut()
                    .screen_update_pixel(xoffs, y, forec.into());
            } else {
                self.io
                    .borrow_mut()
                    .screen_update_pixel(xoffs, y, bgc.into());
            }
        }
    }

    pub fn draw_mcbitmap(&self, x: u32, y: u32, data: u8, scolor: u8, rcolor: u8) {
        for i in 0..4 {
            // Color source
            let cs = ((data >> (i * 2)) & 0x3) as usize;
            // Determine color
            let c = match cs {
                0 => self.bgcolor[0],
                1 => (scolor >> 4) & 0xf,
                2 => scolor & 0xf,
                3 => rcolor,
                _ => unreachable!(), // This case should not happen
            };

            let xoffs = x + 8 - i * 2 + self.horizontal_scroll() as u32;
            // Update pixels
            self.io.borrow_mut().screen_update_pixel(xoffs, y, c.into());
            self.io
                .borrow_mut()
                .screen_update_pixel(xoffs + 1, y, c.into());
        }
    }

    pub fn draw_raster_bitmap_mode(&self) {
        let rstr = self.raster_counter();
        let y = rstr - Vic::FIRST_VISIBLE_LINE as i32;
        if rstr >= Vic::G_FIRST_LINE as i32
            && rstr < Vic::G_LAST_LINE as i32
            && !self.is_screen_off()
        {
            // Draw background
            self.io.borrow_mut().screen_draw_rect(
                Vic::G_FIRST_COL,
                y.try_into().unwrap(),
                Vic::G_RES_X,
                self.bgcolor[0].into(),
            );
            // Draw bitmaps
            for column in 0..Vic::G_COLS {
                let x = Vic::G_FIRST_COL + column * 8;
                let line = rstr - Vic::G_FIRST_LINE as i32;
                let row = line / 8;
                let bitmap_row = line % 8;
                // Retrieve bitmap data
                let data = self.get_bitmap_data(
                    column,
                    row.try_into().unwrap(),
                    bitmap_row.try_into().unwrap(),
                );
                // Retrieve color data
                let scolor = self.get_screen_char(column, row.try_into().unwrap());
                let rcolor = self.get_char_color(column, row.try_into().unwrap());
                // Draw bitmap
                if self.graphic_mode == GraphicsMode::BitmapMode {
                    self.draw_bitmap(x, y.try_into().unwrap(), data, scolor);
                } else {
                    self.draw_mcbitmap(x, y.try_into().unwrap(), data, scolor, rcolor);
                }
            }
        }
    }

    pub fn draw_mcsprite(&self, x: u32, y: u32, sprite: usize, row: u16) {
        let addr = self.get_sprite_ptr(sprite.try_into().unwrap());
        for i in 0..3 {
            let data = self.mem.borrow().vic_read_byte(addr + row * 3 + i as u16);
            for j in 0..4 {
                // Color source
                let cs = (data >> (j * 2)) & 0x3;
                let c = match cs {
                    0 => continue, // transparent, skip drawing
                    1 => self.sprite_shared_colors[0],
                    2 => self.sprite_colors[sprite],
                    3 => self.sprite_shared_colors[1],
                    _ => unreachable!(),
                };

                // Draw if not transparent
                self.io
                    .borrow_mut()
                    .screen_update_pixel(x + i * 8 + 8 - j * 2, y, c.into());
                self.io
                    .borrow_mut()
                    .screen_update_pixel(x + i * 8 + 8 - j * 2 + 1, y, c.into());
            }
        }
    }

    pub fn draw_sprite(&self, x: u32, y: u32, sprite: usize, row: u16) {
        let swid = if self.is_double_width_sprite(sprite) {
            2
        } else {
            1
        };
        let addr = self.get_sprite_ptr(sprite as u32);
        for w in 0..swid {
            for i in 0..3 {
                let data = self.mem.borrow().vic_read_byte(addr + row * 3 + i as u16);
                for j in 0..8 {
                    if (data & (1 << j)) != 0 {
                        let new_x = x + w * 8 * swid + i * 8 * swid + (8 * swid - j * swid);
                        let mut color = self.sprite_colors[sprite];
                        let mut side_border_offset = 0;
                        let mut top_border_offset = 0;
                        let mut btm_border_offset = 0;
                        // Check 38 cols mode
                        if (self.cr2 & (1 << 3)) == 0 {
                            side_border_offset = 8;
                        }
                        // Check 24 line mode
                        if (self.cr1 & (1 << 3)) == 0 {
                            top_border_offset = 2;
                            btm_border_offset = 4;
                        }
                        // Check bounds
                        if new_x <= Vic::G_FIRST_COL + side_border_offset
                            || y < Vic::G_FIRST_COL + top_border_offset
                            || new_x > Vic::G_RES_X + Vic::G_FIRST_COL - side_border_offset
                            || y >= Vic::G_RES_Y + Vic::G_FIRST_COL - btm_border_offset
                        {
                            color = self.border_color;
                        }
                        // Update pixel
                        self.io
                            .borrow_mut()
                            .screen_update_pixel(new_x, y, color.into());
                    }
                }
            }
        }
    }

    pub fn draw_raster_sprites(&self) {
        if self.sprite_enabled != 0 {
            let rstr = self.raster_counter();
            let y = rstr - Vic::FIRST_VISIBLE_LINE as i32;
            let sp_y = rstr - Vic::SPRITES_FIRST_LINE as i32;
            // Loop over sprites in reverse order
            for n in (0..8).rev() {
                let height = if self.is_double_height_sprite(n) {
                    Vic::SPRITE_HEIGHT * 2
                } else {
                    Vic::SPRITE_HEIGHT
                };
                // Check if the sprite is visible
                if self.is_sprite_enabled(n)
                    && sp_y >= self.my[n].into()
                    && sp_y < self.my[n] as i32 + height as i32
                {
                    let mut row = sp_y - self.my[n] as i32;
                    let x = Vic::SPRITES_FIRST_COL + self.sprite_x(n) as u32;
                    if self.is_double_height_sprite(n) {
                        row /= 2;
                    }
                    if self.is_multicolor_sprite(n) {
                        self.draw_mcsprite(x, y.try_into().unwrap(), n, row.try_into().unwrap());
                    } else {
                        self.draw_sprite(x, y.try_into().unwrap(), n, row.try_into().unwrap());
                    }
                }
            }
        }
    }

    pub fn raster_counter_set(&mut self, v: i32) {
        self.raster_c = (v & 0xff) as u8;
        self.cr1 &= 0x7f;
        self.cr1 |= ((v >> 1) & 0x80) as u8;
    }

    pub fn raster_counter(&self) -> i32 {
        (self.raster_c as i32) | (((self.cr1 & 0x80) as i32) << 1)
    }

    pub fn is_screen_off(&self) -> bool {
        (self.cr1 & (1 << 4)) == 0
    }

    pub fn is_bad_line(&self) -> bool {
        let rstr = self.raster_counter();
        rstr >= 0x30 && rstr <= 0xf7 && (rstr & 0x7) == (self.vertical_scroll() & 0x7) as i32
    }

    pub fn raster_irq_enabled(&self) -> bool {
        (self.irq_enabled & 0x01) != 0
    }

    pub fn vertical_scroll(&self) -> u8 {
        self.cr1 & 0x7
    }

    pub fn horizontal_scroll(&self) -> u8 {
        self.cr2 & 0x7
    }

    pub fn is_sprite_enabled(&self, n: usize) -> bool {
        (self.sprite_enabled & (1 << n)) != 0
    }

    pub fn is_background_sprite(&self, n: usize) -> bool {
        (self.sprite_priority & (1 << n)) != 0
    }

    pub fn is_double_width_sprite(&self, n: usize) -> bool {
        (self.sprite_double_width & (1 << n)) != 0
    }

    pub fn is_double_height_sprite(&self, n: usize) -> bool {
        (self.sprite_double_height & (1 << n)) != 0
    }

    pub fn is_multicolor_sprite(&self, n: usize) -> bool {
        (self.sprite_multicolor & (1 << n)) != 0
    }

    pub fn sprite_x(&self, n: usize) -> i32 {
        let mut x = self.mx[n] as i32;
        if (self.msbx & (1 << n)) != 0 {
            x |= 1 << 8;
        }
        x
    }
}
