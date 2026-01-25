// VIC-II 16-color ARGB8888 palette
// Colors based on the C64 wiki color palette

pub const PALETTE: [u32; 16] = [
    0xFF000000, // 0: Black
    0xFFFFFFFF, // 1: White
    0xFF880000, // 2: Red
    0xFFAAFFEE, // 3: Cyan
    0xFFCC44CC, // 4: Purple
    0xFF00CC55, // 5: Green
    0xFF0000AA, // 6: Blue
    0xFFEEEE77, // 7: Yellow
    0xFFDD8855, // 8: Orange
    0xFF664400, // 9: Brown
    0xFFFF7777, // 10: Light Red
    0xFF333333, // 11: Dark Grey
    0xFF777777, // 12: Grey
    0xFFAAFF66, // 13: Light Green
    0xFF0088FF, // 14: Light Blue
    0xFFBBBBBB, // 15: Light Grey
];

/// Get color from palette by index (masked to 4 bits)
#[inline]
pub fn color(index: u8) -> u32 {
    PALETTE[(index & 0x0F) as usize]
}
