// VIC-II Register constants
#![allow(dead_code)] // Many registers for Phase 5+ sprite and advanced graphics features

// Sprite position registers ($D000-$D00F)
pub const REG_SPRITE0_X: u8 = 0x00;
pub const REG_SPRITE0_Y: u8 = 0x01;
// ... sprites 1-7 follow

// Sprite X MSB register
pub const REG_SPRITES_X_MSB: u8 = 0x10;

// Control registers
pub const REG_CTRL1: u8 = 0x11; // Control register 1 (Y scroll, display enable, etc.)
pub const REG_RASTER: u8 = 0x12; // Raster counter (low 8 bits)
pub const REG_LIGHT_PEN_X: u8 = 0x13;
pub const REG_LIGHT_PEN_Y: u8 = 0x14;
pub const REG_SPRITE_ENABLE: u8 = 0x15;
pub const REG_CTRL2: u8 = 0x16; // Control register 2 (X scroll, multicolor, etc.)
pub const REG_SPRITE_Y_EXPAND: u8 = 0x17;
pub const REG_MEMPTR: u8 = 0x18; // Memory pointers (screen/char memory)
pub const REG_IRQ_STATUS: u8 = 0x19; // Interrupt status register
pub const REG_IRQ_ENABLE: u8 = 0x1A; // Interrupt enable register
pub const REG_SPRITE_PRIORITY: u8 = 0x1B;
pub const REG_SPRITE_MULTICOLOR: u8 = 0x1C;
pub const REG_SPRITE_X_EXPAND: u8 = 0x1D;
pub const REG_SPRITE_COLLISION: u8 = 0x1E;
pub const REG_SPRITE_BG_COLLISION: u8 = 0x1F;

// Color registers
pub const REG_BORDER: u8 = 0x20; // Border color
pub const REG_BGCOLOR0: u8 = 0x21; // Background color 0
pub const REG_BGCOLOR1: u8 = 0x22; // Background color 1
pub const REG_BGCOLOR2: u8 = 0x23; // Background color 2
pub const REG_BGCOLOR3: u8 = 0x24; // Background color 3
pub const REG_SPRITE_MULTICOLOR0: u8 = 0x25;
pub const REG_SPRITE_MULTICOLOR1: u8 = 0x26;

// Sprite colors ($D027-$D02E)
pub const REG_SPRITE0_COLOR: u8 = 0x27;

// Control register 1 bits
pub const CTRL1_YSCROLL_MASK: u8 = 0x07;
pub const CTRL1_RSEL: u8 = 0x08; // Row select (24/25 rows)
pub const CTRL1_DEN: u8 = 0x10; // Display enable
pub const CTRL1_BMM: u8 = 0x20; // Bitmap mode
pub const CTRL1_ECM: u8 = 0x40; // Extended color mode
pub const CTRL1_RST8: u8 = 0x80; // Raster bit 8

// Control register 2 bits
pub const CTRL2_XSCROLL_MASK: u8 = 0x07;
pub const CTRL2_CSEL: u8 = 0x08; // Column select (38/40 cols)
pub const CTRL2_MCM: u8 = 0x10; // Multicolor mode
pub const CTRL2_RES: u8 = 0x20; // Reset bit (always 1)

// IRQ bits
pub const IRQ_RASTER: u8 = 0x01;
pub const IRQ_SPRITE_BG: u8 = 0x02;
pub const IRQ_SPRITE_SPRITE: u8 = 0x04;
pub const IRQ_LIGHT_PEN: u8 = 0x08;
pub const IRQ_ANY: u8 = 0x80; // Set if any IRQ is pending

// Memory pointer bits
pub const MEMPTR_CHAR_MASK: u8 = 0x0E; // Character memory bits (shifted left 10)
pub const MEMPTR_SCREEN_MASK: u8 = 0xF0; // Screen memory bits (shifted left 6)
