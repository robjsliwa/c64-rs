use sdl2::keyboard::Keycode;
use std::collections::{HashMap, VecDeque};
use std::vec::Vec;

pub struct Keyboard {
    keyboard_matrix: [u8; 8],
    keymap: HashMap<Keycode, (i32, i32)>,
    charmap: HashMap<char, Vec<Keycode>>,
    key_event_queue: VecDeque<(KeyEvent, Keycode)>,
    next_key_event_at: u32,
}

enum KeyEvent {
    Press,
    Release,
}

impl Keyboard {
    pub fn new() -> Self {
        let mut keyboard = Keyboard {
            keyboard_matrix: [0xff; 8],
            keymap: HashMap::new(),
            charmap: HashMap::new(),
            key_event_queue: VecDeque::new(),
            next_key_event_at: 0,
        };

        // Initilize charmap
        keyboard.charmap.insert('A', vec![Keycode::A]);
        keyboard.charmap.insert('B', vec![Keycode::B]);
        keyboard.charmap.insert('C', vec![Keycode::C]);
        keyboard.charmap.insert('D', vec![Keycode::D]);
        keyboard.charmap.insert('E', vec![Keycode::E]);
        keyboard.charmap.insert('F', vec![Keycode::F]);
        keyboard.charmap.insert('G', vec![Keycode::G]);
        keyboard.charmap.insert('H', vec![Keycode::H]);
        keyboard.charmap.insert('I', vec![Keycode::I]);
        keyboard.charmap.insert('J', vec![Keycode::J]);
        keyboard.charmap.insert('K', vec![Keycode::K]);
        keyboard.charmap.insert('L', vec![Keycode::L]);
        keyboard.charmap.insert('M', vec![Keycode::M]);
        keyboard.charmap.insert('N', vec![Keycode::N]);
        keyboard.charmap.insert('O', vec![Keycode::O]);
        keyboard.charmap.insert('P', vec![Keycode::P]);
        keyboard.charmap.insert('Q', vec![Keycode::Q]);
        keyboard.charmap.insert('R', vec![Keycode::R]);
        keyboard.charmap.insert('S', vec![Keycode::S]);
        keyboard.charmap.insert('T', vec![Keycode::T]);
        keyboard.charmap.insert('U', vec![Keycode::U]);
        keyboard.charmap.insert('V', vec![Keycode::V]);
        keyboard.charmap.insert('W', vec![Keycode::W]);
        keyboard.charmap.insert('X', vec![Keycode::X]);
        keyboard.charmap.insert('Y', vec![Keycode::Y]);
        keyboard.charmap.insert('Z', vec![Keycode::Z]);
        keyboard.charmap.insert('1', vec![Keycode::Num1]);
        keyboard.charmap.insert('2', vec![Keycode::Num2]);
        keyboard.charmap.insert('3', vec![Keycode::Num3]);
        keyboard.charmap.insert('4', vec![Keycode::Num4]);
        keyboard.charmap.insert('5', vec![Keycode::Num5]);
        keyboard.charmap.insert('6', vec![Keycode::Num6]);
        keyboard.charmap.insert('7', vec![Keycode::Num7]);
        keyboard.charmap.insert('8', vec![Keycode::Num8]);
        keyboard.charmap.insert('9', vec![Keycode::Num9]);
        keyboard.charmap.insert('0', vec![Keycode::Num0]);
        keyboard.charmap.insert('\n', vec![Keycode::Return]);
        keyboard.charmap.insert(' ', vec![Keycode::Space]);
        keyboard.charmap.insert(',', vec![Keycode::Comma]);
        keyboard.charmap.insert('.', vec![Keycode::Period]);
        keyboard.charmap.insert('/', vec![Keycode::Slash]);
        keyboard.charmap.insert(';', vec![Keycode::Semicolon]);
        keyboard.charmap.insert('=', vec![Keycode::Equals]);
        keyboard.charmap.insert('-', vec![Keycode::Minus]);
        keyboard.charmap.insert(':', vec![Keycode::Backslash]);
        keyboard.charmap.insert('+', vec![Keycode::LeftBracket]);
        keyboard.charmap.insert('*', vec![Keycode::RightBracket]);
        keyboard.charmap.insert('@', vec![Keycode::Quote]);
        keyboard
            .charmap
            .insert('(', vec![Keycode::LShift, Keycode::Num8]);
        keyboard
            .charmap
            .insert(')', vec![Keycode::LShift, Keycode::Num9]);
        keyboard
            .charmap
            .insert('<', vec![Keycode::LShift, Keycode::Comma]);
        keyboard
            .charmap
            .insert('>', vec![Keycode::LShift, Keycode::Period]);
        keyboard
            .charmap
            .insert('"', vec![Keycode::LShift, Keycode::Num2]);
        keyboard
            .charmap
            .insert('$', vec![Keycode::LShift, Keycode::Num4]);

        // Initialize keymap
        keyboard.keymap.insert(Keycode::A, (1, 2));
        keyboard.keymap.insert(Keycode::B, (3, 4));
        keyboard.keymap.insert(Keycode::C, (2, 4));
        keyboard.keymap.insert(Keycode::D, (2, 2));
        keyboard.keymap.insert(Keycode::E, (1, 6));
        keyboard.keymap.insert(Keycode::F, (2, 5));
        keyboard.keymap.insert(Keycode::G, (3, 2));
        keyboard.keymap.insert(Keycode::H, (3, 5));
        keyboard.keymap.insert(Keycode::I, (4, 1));
        keyboard.keymap.insert(Keycode::J, (4, 2));
        keyboard.keymap.insert(Keycode::K, (4, 5));
        keyboard.keymap.insert(Keycode::L, (5, 2));
        keyboard.keymap.insert(Keycode::M, (4, 4));
        keyboard.keymap.insert(Keycode::N, (4, 7));
        keyboard.keymap.insert(Keycode::O, (4, 6));
        keyboard.keymap.insert(Keycode::P, (5, 1));
        keyboard.keymap.insert(Keycode::Q, (7, 6));
        keyboard.keymap.insert(Keycode::R, (2, 1));
        keyboard.keymap.insert(Keycode::S, (1, 5));
        keyboard.keymap.insert(Keycode::T, (2, 6));
        keyboard.keymap.insert(Keycode::U, (3, 6));
        keyboard.keymap.insert(Keycode::V, (3, 7));
        keyboard.keymap.insert(Keycode::W, (1, 1));
        keyboard.keymap.insert(Keycode::X, (2, 7));
        keyboard.keymap.insert(Keycode::Y, (3, 1));
        keyboard.keymap.insert(Keycode::Z, (1, 4));

        keyboard.keymap.insert(Keycode::Num1, (7, 0));
        keyboard.keymap.insert(Keycode::Num2, (7, 3));
        keyboard.keymap.insert(Keycode::Num3, (1, 0));
        keyboard.keymap.insert(Keycode::Num4, (1, 3));
        keyboard.keymap.insert(Keycode::Num5, (2, 0));
        keyboard.keymap.insert(Keycode::Num6, (2, 3));
        keyboard.keymap.insert(Keycode::Num7, (3, 0));
        keyboard.keymap.insert(Keycode::Num8, (3, 3));
        keyboard.keymap.insert(Keycode::Num9, (4, 0));
        keyboard.keymap.insert(Keycode::Num0, (4, 3));

        keyboard.keymap.insert(Keycode::F1, (0, 4));
        keyboard.keymap.insert(Keycode::F3, (0, 4));
        keyboard.keymap.insert(Keycode::F5, (0, 4));
        keyboard.keymap.insert(Keycode::F7, (0, 4));

        keyboard.keymap.insert(Keycode::Return, (0, 1));
        keyboard.keymap.insert(Keycode::Space, (7, 4));
        keyboard.keymap.insert(Keycode::LShift, (1, 7));
        keyboard.keymap.insert(Keycode::RShift, (6, 4));
        keyboard.keymap.insert(Keycode::Comma, (5, 7));
        keyboard.keymap.insert(Keycode::Period, (5, 4));
        keyboard.keymap.insert(Keycode::Slash, (6, 7));
        keyboard.keymap.insert(Keycode::Semicolon, (6, 2));
        keyboard.keymap.insert(Keycode::Equals, (6, 5));
        keyboard.keymap.insert(Keycode::Backspace, (0, 0));
        keyboard.keymap.insert(Keycode::Minus, (5, 3));

        keyboard.keymap.insert(Keycode::Backslash, (5, 5));
        keyboard.keymap.insert(Keycode::LeftBracket, (5, 0));
        keyboard.keymap.insert(Keycode::RightBracket, (6, 1));
        keyboard.keymap.insert(Keycode::Quote, (5, 6));
        keyboard.keymap.insert(Keycode::LGui, (7, 5)); // Commodore key

        keyboard
    }

    pub fn keyboard_matrix_row(&self, col: usize) -> u8 {
        self.keyboard_matrix[col]
    }

    pub fn handle_keydown(&mut self, key: Keycode) {
        if let Some(&(row, col)) = self.keymap.get(&key) {
            let mask = !(1 << col);
            self.keyboard_matrix[row as usize] &= mask;
        }
    }

    pub fn handle_keyup(&mut self, key: Keycode) {
        if let Some(&(row, col)) = self.keymap.get(&key) {
            let mask = 1 << col;
            self.keyboard_matrix[row as usize] |= mask;
        }
    }

    pub fn queue_key_event(&mut self, event: KeyEvent, key: Keycode) {
        self.key_event_queue.push_back((event, key));
    }

    pub fn process_key_event(&mut self) {
        if let Some((event, key)) = self.key_event_queue.pop_front() {
            match event {
                KeyEvent::Press => self.handle_keydown(key),
                KeyEvent::Release => self.handle_keyup(key),
            }
        }
    }
}
