// VIC-II Sprite state management

use std::cell::Cell;

/// Sprite state for VIC-II
/// The VIC-II supports 8 hardware sprites (also called MOBs - Movable Object Blocks)
pub struct SpriteState {
    /// 9-bit X positions for each sprite (0-511)
    /// Low 8 bits from $D000/$D002/etc, bit 8 from $D010
    pub x: [u16; 8],

    /// 8-bit Y positions for each sprite (0-255)
    /// From $D001/$D003/etc
    pub y: [u8; 8],

    /// Individual sprite colors (lower 4 bits only)
    /// From $D027-$D02E
    pub color: [u8; 8],

    /// Sprite multicolor 0 ($D025)
    /// Used when sprite is in multicolor mode (bit pattern 01)
    pub multicolor0: u8,

    /// Sprite multicolor 1 ($D026)
    /// Used when sprite is in multicolor mode (bit pattern 11)
    pub multicolor1: u8,

    /// Sprite enable bitmask ($D015)
    /// Bit N = 1 means sprite N is enabled
    pub enabled: u8,

    /// Sprite-to-background priority bitmask ($D01B)
    /// Bit N = 1 means sprite N appears behind background
    pub priority: u8,

    /// Sprite multicolor mode bitmask ($D01C)
    /// Bit N = 1 means sprite N uses multicolor mode
    pub multicolor: u8,

    /// Sprite X expansion bitmask ($D01D)
    /// Bit N = 1 means sprite N is double width
    pub expand_x: u8,

    /// Sprite Y expansion bitmask ($D017)
    /// Bit N = 1 means sprite N is double height
    pub expand_y: u8,

    /// Sprite-sprite collision register ($D01E)
    /// Set when sprites collide, cleared on read
    /// Uses Cell for interior mutability (clear on read behavior)
    pub collision_sprite: Cell<u8>,

    /// Sprite-background collision register ($D01F)
    /// Set when sprite collides with background, cleared on read
    /// Uses Cell for interior mutability (clear on read behavior)
    pub collision_bg: Cell<u8>,
}

impl Default for SpriteState {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteState {
    pub fn new() -> Self {
        SpriteState {
            x: [0; 8],
            y: [0; 8],
            color: [0; 8],
            multicolor0: 0,
            multicolor1: 0,
            enabled: 0,
            priority: 0,
            multicolor: 0,
            expand_x: 0,
            expand_y: 0,
            collision_sprite: Cell::new(0),
            collision_bg: Cell::new(0),
        }
    }

    /// Reset sprite state to defaults
    pub fn reset(&mut self) {
        self.x = [0; 8];
        self.y = [0; 8];
        self.color = [0; 8];
        self.multicolor0 = 0;
        self.multicolor1 = 0;
        self.enabled = 0;
        self.priority = 0;
        self.multicolor = 0;
        self.expand_x = 0;
        self.expand_y = 0;
        self.collision_sprite.set(0);
        self.collision_bg.set(0);
    }

    /// Read and clear sprite-sprite collision register
    pub fn read_collision_sprite(&self) -> u8 {
        let value = self.collision_sprite.get();
        self.collision_sprite.set(0);
        value
    }

    /// Read and clear sprite-background collision register
    pub fn read_collision_bg(&self) -> u8 {
        let value = self.collision_bg.get();
        self.collision_bg.set(0);
        value
    }

    /// Set sprite-sprite collision bit
    #[cfg(test)]
    pub fn set_collision_sprite(&self, sprite_num: usize) {
        let current = self.collision_sprite.get();
        self.collision_sprite.set(current | (1 << sprite_num));
    }

    /// Set sprite-background collision bit
    #[cfg(test)]
    pub fn set_collision_bg(&self, sprite_num: usize) {
        let current = self.collision_bg.get();
        self.collision_bg.set(current | (1 << sprite_num));
    }

    /// Check if a sprite is enabled
    #[inline]
    pub fn is_enabled(&self, sprite_num: usize) -> bool {
        (self.enabled & (1 << sprite_num)) != 0
    }

    /// Check if a sprite has priority (appears behind background)
    #[inline]
    pub fn has_priority_behind(&self, sprite_num: usize) -> bool {
        (self.priority & (1 << sprite_num)) != 0
    }

    /// Check if a sprite is in multicolor mode
    #[inline]
    pub fn is_multicolor(&self, sprite_num: usize) -> bool {
        (self.multicolor & (1 << sprite_num)) != 0
    }

    /// Check if a sprite has X expansion
    #[inline]
    pub fn is_expanded_x(&self, sprite_num: usize) -> bool {
        (self.expand_x & (1 << sprite_num)) != 0
    }

    /// Check if a sprite has Y expansion
    #[inline]
    pub fn is_expanded_y(&self, sprite_num: usize) -> bool {
        (self.expand_y & (1 << sprite_num)) != 0
    }

    /// Get sprite height (21 or 42 if Y-expanded)
    #[inline]
    pub fn get_height(&self, sprite_num: usize) -> u16 {
        if self.is_expanded_y(sprite_num) {
            42
        } else {
            21
        }
    }

    /// Get sprite width in pixels (24 or 48 if X-expanded)
    #[inline]
    #[allow(dead_code)]
    pub fn get_width(&self, sprite_num: usize) -> u16 {
        if self.is_expanded_x(sprite_num) {
            48
        } else {
            24
        }
    }

    /// Update X MSB register ($D010) - sets bit 8 of each sprite's X position
    pub fn update_x_msb(&mut self, value: u8) {
        for i in 0..8 {
            if value & (1 << i) != 0 {
                self.x[i] |= 0x100;
            } else {
                self.x[i] &= 0x0FF;
            }
        }
    }

    /// Get X MSB register value
    pub fn get_x_msb(&self) -> u8 {
        let mut result = 0u8;
        for i in 0..8 {
            if self.x[i] & 0x100 != 0 {
                result |= 1 << i;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_state_new() {
        let state = SpriteState::new();
        assert_eq!(state.enabled, 0);
        assert_eq!(state.x, [0; 8]);
        assert_eq!(state.y, [0; 8]);
        assert_eq!(state.collision_sprite.get(), 0);
        assert_eq!(state.collision_bg.get(), 0);
    }

    #[test]
    fn test_sprite_state_reset() {
        let mut state = SpriteState::new();
        state.enabled = 0xFF;
        state.x[0] = 100;
        state.y[0] = 50;
        state.collision_sprite.set(0x03);

        state.reset();

        assert_eq!(state.enabled, 0);
        assert_eq!(state.x[0], 0);
        assert_eq!(state.y[0], 0);
        assert_eq!(state.collision_sprite.get(), 0);
    }

    #[test]
    fn test_sprite_enabled() {
        let mut state = SpriteState::new();
        assert!(!state.is_enabled(0));
        assert!(!state.is_enabled(7));

        state.enabled = 0b10000001; // Sprites 0 and 7 enabled
        assert!(state.is_enabled(0));
        assert!(!state.is_enabled(1));
        assert!(state.is_enabled(7));
    }

    #[test]
    fn test_sprite_priority() {
        let mut state = SpriteState::new();
        assert!(!state.has_priority_behind(0));

        state.priority = 0b00000100; // Sprite 2 behind background
        assert!(!state.has_priority_behind(0));
        assert!(state.has_priority_behind(2));
    }

    #[test]
    fn test_sprite_multicolor() {
        let mut state = SpriteState::new();
        assert!(!state.is_multicolor(0));

        state.multicolor = 0b00001000; // Sprite 3 is multicolor
        assert!(!state.is_multicolor(0));
        assert!(state.is_multicolor(3));
    }

    #[test]
    fn test_sprite_expansion() {
        let mut state = SpriteState::new();
        assert!(!state.is_expanded_x(0));
        assert!(!state.is_expanded_y(0));
        assert_eq!(state.get_width(0), 24);
        assert_eq!(state.get_height(0), 21);

        state.expand_x = 0b00000001; // Sprite 0 X-expanded
        state.expand_y = 0b00000010; // Sprite 1 Y-expanded

        assert!(state.is_expanded_x(0));
        assert!(!state.is_expanded_y(0));
        assert_eq!(state.get_width(0), 48);
        assert_eq!(state.get_height(0), 21);

        assert!(!state.is_expanded_x(1));
        assert!(state.is_expanded_y(1));
        assert_eq!(state.get_width(1), 24);
        assert_eq!(state.get_height(1), 42);
    }

    #[test]
    fn test_x_msb_register() {
        let mut state = SpriteState::new();

        // Set sprite 0 X position to 300 (0x12C)
        state.x[0] = 0x2C; // Low 8 bits
        state.update_x_msb(0b00000001); // Set MSB for sprite 0

        assert_eq!(state.x[0], 0x12C);
        assert_eq!(state.get_x_msb(), 0b00000001);

        // Set sprite 7 X position to 400 (0x190)
        state.x[7] = 0x90; // Low 8 bits
        state.update_x_msb(0b10000001); // Set MSB for sprites 0 and 7

        assert_eq!(state.x[0], 0x12C);
        assert_eq!(state.x[7], 0x190);
        assert_eq!(state.get_x_msb(), 0b10000001);
    }

    #[test]
    fn test_x_msb_clear() {
        let mut state = SpriteState::new();

        // Set X position with MSB
        state.x[0] = 0x12C;
        state.update_x_msb(0b00000001);
        assert_eq!(state.x[0], 0x12C);

        // Clear MSB
        state.update_x_msb(0b00000000);
        assert_eq!(state.x[0], 0x2C); // MSB cleared, only low byte remains
    }

    #[test]
    fn test_collision_clear_on_read() {
        let state = SpriteState::new();

        // Set collision bits
        state.set_collision_sprite(0);
        state.set_collision_sprite(2);
        assert_eq!(state.collision_sprite.get(), 0b00000101);

        // First read returns value and clears
        let value = state.read_collision_sprite();
        assert_eq!(value, 0b00000101);

        // Second read returns 0
        let value = state.read_collision_sprite();
        assert_eq!(value, 0);
    }

    #[test]
    fn test_collision_bg_clear_on_read() {
        let state = SpriteState::new();

        state.set_collision_bg(1);
        state.set_collision_bg(5);
        assert_eq!(state.collision_bg.get(), 0b00100010);

        let value = state.read_collision_bg();
        assert_eq!(value, 0b00100010);

        let value = state.read_collision_bg();
        assert_eq!(value, 0);
    }
}
