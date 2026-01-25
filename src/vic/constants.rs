// VIC-II PAL timing constants
#![allow(dead_code)] // Many constants for Phase 5+ features

// PAL timing
pub const SCREEN_LINES: u16 = 312;
pub const CYCLES_PER_LINE: u64 = 63;
pub const BAD_LINE_CYCLES: u64 = 23;

// Display dimensions (visible area)
pub const VISIBLE_WIDTH: u32 = 403;
pub const VISIBLE_HEIGHT: u32 = 284;

// Active display area (where characters/graphics appear)
pub const DISPLAY_WIDTH: u32 = 320;
pub const DISPLAY_HEIGHT: u32 = 200;

// Visible line boundaries
pub const FIRST_VISIBLE_LINE: u16 = 14;
pub const LAST_VISIBLE_LINE: u16 = 298;

// Display area boundaries (where 320x200 content appears)
pub const FIRST_DISPLAY_LINE: u16 = 56;
pub const LAST_DISPLAY_LINE: u16 = 256;

// Display offsets within visible area
pub const DISPLAY_START_X: u32 = 42;
pub const DISPLAY_START_Y: u32 = 42;

// Frame rate
pub const FRAME_RATE: f64 = 50.125;

// Character mode constants
pub const CHAR_COLS: u32 = 40;
pub const CHAR_ROWS: u32 = 25;
pub const CHAR_WIDTH: u32 = 8;
pub const CHAR_HEIGHT: u32 = 8;

// Sprite constants
pub const SPRITE_WIDTH: u32 = 24;
pub const SPRITE_HEIGHT: u32 = 21;
pub const SPRITE_BYTES_PER_LINE: u32 = 3;
pub const SPRITE_DATA_SIZE: u32 = 63; // 21 lines * 3 bytes
pub const NUM_SPRITES: usize = 8;

// Sprite pointer offset from screen memory
pub const SPRITE_POINTER_OFFSET: u16 = 0x3F8;
