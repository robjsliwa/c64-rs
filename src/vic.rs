pub struct Vic {}

impl Vic {
    pub const SCREEN_LINES: u32 = 312;
    pub const SCREEN_COLUMNS: u32 = 504;
    pub const VISIBLE_SCREEN_WIDTH: u32 = 403;
    pub const VISIBLE_SCREEN_HEIGHT: u32 = 284;
    pub const FIRST_VISIBLE_LINE: u32 = 14;
    pub const LAST_VISIBLE_LINE: u32 = 298;
    pub const LINE_CYCLES: u32 = 63;
    pub const BAD_LINE_CYCLES: u32 = 23;
    pub const K_REFRESH_RATE: f64 = 1f64 / 50.125;
    pub const SPRITE_PTRS_OFFSET: u32 = 0x3f8;
}
