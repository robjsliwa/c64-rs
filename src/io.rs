use super::cpu::Cpu;
use crate::vic::constants::{VISIBLE_HEIGHT, VISIBLE_WIDTH};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::vec::Vec;

// Window scale factor (2x)
const SCALE: u32 = 2;
const WINDOW_WIDTH: u32 = VISIBLE_WIDTH * SCALE;
const WINDOW_HEIGHT: u32 = VISIBLE_HEIGHT * SCALE;
const FRAME_BUFFER_SIZE: usize = (VISIBLE_WIDTH * VISIBLE_HEIGHT) as usize;

// Target frame duration for ~50Hz
const FRAME_DURATION_NS: u64 = 19_950_125; // ~50.125 Hz

pub struct IO {
    cpu: Rc<RefCell<Cpu>>,
    keyboard_matrix: [u8; 8],
    keymap: HashMap<Keycode, (i32, i32)>,
    charmap: HashMap<char, Vec<Keycode>>,
    key_event_queue: VecDeque<(KeyEvent, Keycode)>,
    next_key_event_at: u32,
    event_pump: EventPump,
    retval: bool,
    renderer: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    frame_buffer: Box<[u32; FRAME_BUFFER_SIZE]>,
    frame_start: Instant,
}

pub(crate) enum KeyEvent {
    Press,
    Release,
}

impl IO {
    pub const WAIT_DURATION: u32 = 18000;

    pub fn new(cpu: Rc<RefCell<Cpu>>) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Commodore C64", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        let frame_buffer = Box::new([0u32; FRAME_BUFFER_SIZE]);

        let event_pump = sdl_context.event_pump()?;

        let mut io = IO {
            cpu,
            keyboard_matrix: [0xff; 8],
            keymap: HashMap::new(),
            charmap: HashMap::new(),
            key_event_queue: VecDeque::new(),
            next_key_event_at: 0,
            event_pump,
            retval: true,
            renderer: canvas,
            texture_creator,
            frame_buffer,
            frame_start: Instant::now(),
        };

        // Initialize charmap
        io.charmap.insert('A', vec![Keycode::A]);
        io.charmap.insert('B', vec![Keycode::B]);
        io.charmap.insert('C', vec![Keycode::C]);
        io.charmap.insert('D', vec![Keycode::D]);
        io.charmap.insert('E', vec![Keycode::E]);
        io.charmap.insert('F', vec![Keycode::F]);
        io.charmap.insert('G', vec![Keycode::G]);
        io.charmap.insert('H', vec![Keycode::H]);
        io.charmap.insert('I', vec![Keycode::I]);
        io.charmap.insert('J', vec![Keycode::J]);
        io.charmap.insert('K', vec![Keycode::K]);
        io.charmap.insert('L', vec![Keycode::L]);
        io.charmap.insert('M', vec![Keycode::M]);
        io.charmap.insert('N', vec![Keycode::N]);
        io.charmap.insert('O', vec![Keycode::O]);
        io.charmap.insert('P', vec![Keycode::P]);
        io.charmap.insert('Q', vec![Keycode::Q]);
        io.charmap.insert('R', vec![Keycode::R]);
        io.charmap.insert('S', vec![Keycode::S]);
        io.charmap.insert('T', vec![Keycode::T]);
        io.charmap.insert('U', vec![Keycode::U]);
        io.charmap.insert('V', vec![Keycode::V]);
        io.charmap.insert('W', vec![Keycode::W]);
        io.charmap.insert('X', vec![Keycode::X]);
        io.charmap.insert('Y', vec![Keycode::Y]);
        io.charmap.insert('Z', vec![Keycode::Z]);
        io.charmap.insert('1', vec![Keycode::Num1]);
        io.charmap.insert('2', vec![Keycode::Num2]);
        io.charmap.insert('3', vec![Keycode::Num3]);
        io.charmap.insert('4', vec![Keycode::Num4]);
        io.charmap.insert('5', vec![Keycode::Num5]);
        io.charmap.insert('6', vec![Keycode::Num6]);
        io.charmap.insert('7', vec![Keycode::Num7]);
        io.charmap.insert('8', vec![Keycode::Num8]);
        io.charmap.insert('9', vec![Keycode::Num9]);
        io.charmap.insert('0', vec![Keycode::Num0]);
        io.charmap.insert('\n', vec![Keycode::Return]);
        io.charmap.insert(' ', vec![Keycode::Space]);
        io.charmap.insert(',', vec![Keycode::Comma]);
        io.charmap.insert('.', vec![Keycode::Period]);
        io.charmap.insert('/', vec![Keycode::Slash]);
        io.charmap.insert(';', vec![Keycode::Semicolon]);
        io.charmap.insert('=', vec![Keycode::Equals]);
        io.charmap.insert('-', vec![Keycode::Minus]);
        io.charmap.insert(':', vec![Keycode::Backslash]);
        io.charmap.insert('+', vec![Keycode::LeftBracket]);
        io.charmap.insert('*', vec![Keycode::RightBracket]);
        io.charmap.insert('@', vec![Keycode::Quote]);
        io.charmap.insert('(', vec![Keycode::LShift, Keycode::Num8]);
        io.charmap.insert(')', vec![Keycode::LShift, Keycode::Num9]);
        io.charmap
            .insert('<', vec![Keycode::LShift, Keycode::Comma]);
        io.charmap
            .insert('>', vec![Keycode::LShift, Keycode::Period]);
        io.charmap.insert('"', vec![Keycode::LShift, Keycode::Num2]);
        io.charmap.insert('$', vec![Keycode::LShift, Keycode::Num4]);

        // Initialize keymap
        io.keymap.insert(Keycode::A, (1, 2));
        io.keymap.insert(Keycode::B, (3, 4));
        io.keymap.insert(Keycode::C, (2, 4));
        io.keymap.insert(Keycode::D, (2, 2));
        io.keymap.insert(Keycode::E, (1, 6));
        io.keymap.insert(Keycode::F, (2, 5));
        io.keymap.insert(Keycode::G, (3, 2));
        io.keymap.insert(Keycode::H, (3, 5));
        io.keymap.insert(Keycode::I, (4, 1));
        io.keymap.insert(Keycode::J, (4, 2));
        io.keymap.insert(Keycode::K, (4, 5));
        io.keymap.insert(Keycode::L, (5, 2));
        io.keymap.insert(Keycode::M, (4, 4));
        io.keymap.insert(Keycode::N, (4, 7));
        io.keymap.insert(Keycode::O, (4, 6));
        io.keymap.insert(Keycode::P, (5, 1));
        io.keymap.insert(Keycode::Q, (7, 6));
        io.keymap.insert(Keycode::R, (2, 1));
        io.keymap.insert(Keycode::S, (1, 5));
        io.keymap.insert(Keycode::T, (2, 6));
        io.keymap.insert(Keycode::U, (3, 6));
        io.keymap.insert(Keycode::V, (3, 7));
        io.keymap.insert(Keycode::W, (1, 1));
        io.keymap.insert(Keycode::X, (2, 7));
        io.keymap.insert(Keycode::Y, (3, 1));
        io.keymap.insert(Keycode::Z, (1, 4));

        io.keymap.insert(Keycode::Num1, (7, 0));
        io.keymap.insert(Keycode::Num2, (7, 3));
        io.keymap.insert(Keycode::Num3, (1, 0));
        io.keymap.insert(Keycode::Num4, (1, 3));
        io.keymap.insert(Keycode::Num5, (2, 0));
        io.keymap.insert(Keycode::Num6, (2, 3));
        io.keymap.insert(Keycode::Num7, (3, 0));
        io.keymap.insert(Keycode::Num8, (3, 3));
        io.keymap.insert(Keycode::Num9, (4, 0));
        io.keymap.insert(Keycode::Num0, (4, 3));

        io.keymap.insert(Keycode::F1, (0, 4));
        io.keymap.insert(Keycode::F3, (0, 4));
        io.keymap.insert(Keycode::F5, (0, 4));
        io.keymap.insert(Keycode::F7, (0, 4));

        io.keymap.insert(Keycode::Return, (0, 1));
        io.keymap.insert(Keycode::Space, (7, 4));
        io.keymap.insert(Keycode::LShift, (1, 7));
        io.keymap.insert(Keycode::RShift, (6, 4));
        io.keymap.insert(Keycode::Comma, (5, 7));
        io.keymap.insert(Keycode::Period, (5, 4));
        io.keymap.insert(Keycode::Slash, (6, 7));
        io.keymap.insert(Keycode::Semicolon, (6, 2));
        io.keymap.insert(Keycode::Equals, (6, 5));
        io.keymap.insert(Keycode::Backspace, (0, 0));
        io.keymap.insert(Keycode::Minus, (5, 3));

        io.keymap.insert(Keycode::Backslash, (5, 5));
        io.keymap.insert(Keycode::LeftBracket, (5, 0));
        io.keymap.insert(Keycode::RightBracket, (6, 1));
        io.keymap.insert(Keycode::Quote, (5, 6));
        io.keymap.insert(Keycode::LGui, (7, 5)); // Commodore key

        Ok(io)
    }

    pub fn keyboard_matrix_row(&self, col: usize) -> u8 {
        // The keyboard matrix is stored by row, but we need to read by column
        // Each row has 8 bits representing columns 0-7
        // We need to build a byte where each bit represents a row for the given column
        let mut result = 0xFF; // All keys released (active low)
        
        for row in 0..8 {
            // Check if the bit for this column is clear in this row (key pressed)
            if (self.keyboard_matrix[row] & (1 << col)) == 0 {
                // Key is pressed, clear the corresponding row bit
                result &= !(1 << row);
            }
        }
        
        result
    }

    pub fn handle_keydown(&mut self, key: Keycode) {
        if let Some(&(col, row)) = self.keymap.get(&key) {
            // Keymap stores (col, row) not (row, col)
            // Clear the bit at [row][col] in the matrix
            let mask = !(1 << col);
            self.keyboard_matrix[row as usize] &= mask;
        }
    }

    pub fn handle_keyup(&mut self, key: Keycode) {
        if let Some(&(col, row)) = self.keymap.get(&key) {
            // Keymap stores (col, row) not (row, col)
            // Set the bit at [row][col] in the matrix
            let mask = 1 << col;
            self.keyboard_matrix[row as usize] |= mask;
        }
    }

    #[allow(dead_code)] // Used for programmatic keyboard input (auto-typing BASIC programs)
    pub fn queue_key_event(&mut self, event: KeyEvent, key: Keycode) {
        self.key_event_queue.push_back((event, key));
    }

    #[allow(dead_code)] // Used for programmatic keyboard input
    pub fn type_character(&mut self, character: char) {
        if let Some(keycodes) = self.charmap.get(&character).cloned() {
            for keycode in keycodes {
                self.queue_key_event(KeyEvent::Press, keycode);
                self.queue_key_event(KeyEvent::Release, keycode);
            }
        }
    }

    pub fn process_events(&mut self) {
        let events: Vec<sdl2::event::Event> = self.event_pump.poll_iter().collect();

        for event in events {
            match event {
                sdl2::event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => self.handle_keydown(keycode),
                sdl2::event::Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => self.handle_keyup(keycode),
                sdl2::event::Event::Quit { .. } => {
                    self.retval = false;
                }
                _ => {}
            }
        }

        // Process fake keystrokes if any
        if !self.key_event_queue.is_empty() && self.cpu.borrow().cycles() > self.next_key_event_at {
            if let Some((event, keycode)) = self.key_event_queue.pop_front() {
                match event {
                    KeyEvent::Press => self.handle_keydown(keycode),
                    KeyEvent::Release => self.handle_keyup(keycode),
                }
            }
            self.next_key_event_at = self.cpu.borrow().cycles() + Self::WAIT_DURATION;
        }
    }

    pub fn step(&self) -> bool {
        self.retval
    }

    /// Set a single pixel in the frame buffer
    #[inline]
    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < VISIBLE_WIDTH && y < VISIBLE_HEIGHT {
            let offset = (y * VISIBLE_WIDTH + x) as usize;
            self.frame_buffer[offset] = color;
        }
    }

    /// Fill a rectangle in the frame buffer
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x_end = (x + w).min(VISIBLE_WIDTH);
        let y_end = (y + h).min(VISIBLE_HEIGHT);

        for py in y..y_end {
            let row_offset = (py * VISIBLE_WIDTH) as usize;
            for px in x..x_end {
                self.frame_buffer[row_offset + px as usize] = color;
            }
        }
    }

    /// Refresh the display - upload frame buffer to texture and present
    pub fn refresh(&mut self) {
        // Create texture, render, and present in a block to ensure texture is dropped
        {
            // Create a streaming texture
            let mut texture = self
                .texture_creator
                .create_texture_streaming(
                    PixelFormatEnum::ARGB8888,
                    VISIBLE_WIDTH,
                    VISIBLE_HEIGHT,
                )
                .expect("Failed to create texture");

            // Upload frame buffer to texture
            texture
                .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for y in 0..VISIBLE_HEIGHT as usize {
                        for x in 0..VISIBLE_WIDTH as usize {
                            let src_offset = y * VISIBLE_WIDTH as usize + x;
                            let dst_offset = y * pitch + x * 4;
                            let pixel = self.frame_buffer[src_offset];

                            // ARGB8888 format (little-endian byte order: B, G, R, A)
                            buffer[dst_offset] = pixel as u8; // B
                            buffer[dst_offset + 1] = (pixel >> 8) as u8; // G
                            buffer[dst_offset + 2] = (pixel >> 16) as u8; // R
                            buffer[dst_offset + 3] = (pixel >> 24) as u8; // A
                        }
                    }
                })
                .expect("Failed to lock texture");

            // Copy texture to renderer (with scaling)
            self.renderer
                .copy(&texture, None, None)
                .expect("Failed to copy texture");

            // Present
            self.renderer.present();
        }

        // VSync timing (after texture is dropped)
        self.vsync();
    }

    /// Wait for vsync timing (~50Hz)
    fn vsync(&mut self) {
        let elapsed = self.frame_start.elapsed();
        let target = Duration::from_nanos(FRAME_DURATION_NS);

        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }

        self.frame_start = Instant::now();
    }
}
