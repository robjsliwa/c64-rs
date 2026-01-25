use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Weak;

use crate::cpu::Cpu;
use crate::io::IO;
use crate::memory::Memory;

#[derive(Debug)]
pub enum LoaderError {
    #[allow(dead_code)]
    IoError(std::io::Error),
    InvalidProgram,
    NoMemory,
    #[allow(dead_code)]
    NoCpu, // Reserved for future use
    #[allow(dead_code)]
    NoKeyboard, // Reserved for future use
    UnknownFileType,
}

impl From<std::io::Error> for LoaderError {
    fn from(err: std::io::Error) -> Self {
        LoaderError::IoError(err)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LoaderState {
    Idle,
    WaitingForBasic, // Wait for BASIC to be ready
    #[allow(dead_code)]
    LoadingPrg, // Loading PRG into memory (reserved for future streaming)
    TypingBasic,     // Typing BAS file
    Running,         // Program loaded, running
    Complete,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FileType {
    Prg,
    Bas,
    Unknown,
}

pub struct Loader {
    state: LoaderState,
    file_type: FileType,
    data: Vec<u8>,
    #[allow(dead_code)]
    position: usize, // Reserved for future use (streaming large files)

    // For BAS files
    char_queue: VecDeque<char>,

    // References
    memory: Weak<RefCell<Memory>>,
    cpu: Weak<RefCell<Cpu>>,
    keyboard: Weak<RefCell<IO>>,

    // Timing for character typing
    next_char_at: u32,
}

impl Loader {
    // BASIC main loop address (after printing READY.)
    const BASIC_READY_PC: u16 = 0xA65C;

    // Delay between typed characters (in cycles)
    const CHAR_DELAY: u32 = IO::WAIT_DURATION;

    /// Load a file from disk
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self, LoaderError> {
        let path = path.as_ref();
        let data = std::fs::read(path)?;

        let file_type = Self::detect_type(path, &data);

        if file_type == FileType::Unknown {
            return Err(LoaderError::UnknownFileType);
        }

        Ok(Self {
            state: LoaderState::Idle,
            file_type,
            data,
            position: 0,
            char_queue: VecDeque::new(),
            memory: Weak::new(),
            cpu: Weak::new(),
            keyboard: Weak::new(),
            next_char_at: 0,
        })
    }

    /// Detect file type from extension and content
    fn detect_type(path: &Path, data: &[u8]) -> FileType {
        // Check extension first
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            match ext.as_str() {
                "prg" => return FileType::Prg,
                "bas" => return FileType::Bas,
                _ => {}
            }
        }

        // Check content for PRG (should have valid load address)
        if data.len() >= 2 {
            let load_addr = u16::from_le_bytes([data[0], data[1]]);
            // Common PRG load addresses
            if load_addr == 0x0801 || load_addr == 0x1000 || load_addr >= 0xC000 {
                return FileType::Prg;
            }
        }

        // Check if content looks like BASIC text
        if data.len() > 0 && data.iter().all(|&b| b.is_ascii() || b == 0x0D || b == 0x0A) {
            return FileType::Bas;
        }

        FileType::Unknown
    }

    /// Set references to other components
    pub fn set_references(
        &mut self,
        memory: Weak<RefCell<Memory>>,
        cpu: Weak<RefCell<Cpu>>,
        keyboard: Weak<RefCell<IO>>,
    ) {
        self.memory = memory;
        self.cpu = cpu;
        self.keyboard = keyboard;
    }

    /// Check if BASIC is ready
    fn is_basic_ready(&self) -> bool {
        if let Some(cpu) = self.cpu.upgrade() {
            let pc = cpu.borrow().pc;
            pc == Self::BASIC_READY_PC
        } else {
            false
        }
    }

    /// Load a PRG file into memory
    fn load_prg(&mut self) -> Result<(), LoaderError> {
        if self.data.len() < 3 {
            return Err(LoaderError::InvalidProgram);
        }

        // Get load address
        let load_addr = u16::from_le_bytes([self.data[0], self.data[1]]);

        let mem = self.memory.upgrade().ok_or(LoaderError::NoMemory)?;
        let mut mem = mem.borrow_mut();

        // Copy data to memory
        for (i, byte) in self.data[2..].iter().enumerate() {
            mem.write_byte_no_io(load_addr.wrapping_add(i as u16), *byte);
        }

        // Calculate end address
        let end_addr = load_addr.wrapping_add((self.data.len() - 2) as u16);

        // If BASIC program, set up pointers
        if load_addr == 0x0801 {
            // Set BASIC zero-page pointers correctly for a loaded program
            // $2B-$2C: Start of BASIC text
            mem.write_word(0x002B, load_addr);

            // $2D-$2E: End of BASIC text / Start of variables
            mem.write_word(0x002D, end_addr);

            // $2F-$30: Start of arrays
            mem.write_word(0x002F, end_addr);

            // $31-$32: End of arrays / Start of strings
            mem.write_word(0x0031, end_addr);

            // $33-$34: End of strings / Start of free memory
            mem.write_word(0x0033, end_addr);

            // Drop the borrow before queuing commands
            drop(mem);

            // For PRG BASIC programs that are already tokenized,
            // we just type RUN to execute them
            self.char_queue.push_back('R');
            self.char_queue.push_back('U');
            self.char_queue.push_back('N');
            self.char_queue.push_back('\n');
            self.state = LoaderState::WaitingForBasic;
        } else {
            // ML program - set PC directly
            drop(mem);
            if let Some(cpu) = self.cpu.upgrade() {
                cpu.borrow_mut().pc = load_addr;
            }
            self.state = LoaderState::Running;
        }

        Ok(())
    }

    /// Load a BAS file (prepare for typing)
    fn load_bas(&mut self) -> Result<(), LoaderError> {
        // Convert file data to string
        let text = String::from_utf8_lossy(&self.data);

        // Process each line
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Skip REM lines and NEW command (they're just comments/utilities)
            if line.starts_with("REM") || line.eq_ignore_ascii_case("NEW") {
                continue;
            }

            // Queue each character
            for ch in line.chars() {
                // Only uppercase letters A-Z, preserve special characters
                let ch = if ch.is_ascii_alphabetic() {
                    ch.to_ascii_uppercase()
                } else {
                    ch
                };
                self.char_queue.push_back(ch);
            }

            // Add RETURN to enter the line
            self.char_queue.push_back('\n');
        }

        // Add RUN command
        self.char_queue.push_back('R');
        self.char_queue.push_back('U');
        self.char_queue.push_back('N');
        self.char_queue.push_back('\n');

        self.state = LoaderState::WaitingForBasic;
        Ok(())
    }

    /// Type the next character from the queue
    fn type_next_char(&mut self) {
        if let Some(ch) = self.char_queue.pop_front() {
            if let Some(keyboard) = self.keyboard.upgrade() {
                keyboard.borrow_mut().type_character(ch);
            }
        } else {
            self.state = LoaderState::Complete;
        }
    }

    /// Emulate one step of the loader
    pub fn emulate(&mut self) -> bool {
        match self.state {
            LoaderState::Idle => {
                eprintln!("[LOADER] Starting load, type={:?}", self.file_type);
                // Start loading
                match self.file_type {
                    FileType::Prg => {
                        if let Err(e) = self.load_prg() {
                            eprintln!("Error loading PRG: {:?}", e);
                            self.state = LoaderState::Complete;
                        } else {
                            eprintln!("[LOADER] PRG loaded, state={:?}", self.state);
                        }
                    }
                    FileType::Bas => {
                        if let Err(e) = self.load_bas() {
                            eprintln!("Error loading BAS: {:?}", e);
                            self.state = LoaderState::Complete;
                        } else {
                            eprintln!(
                                "[LOADER] BAS loaded, {} chars queued, state={:?}",
                                self.char_queue.len(),
                                self.state
                            );
                        }
                    }
                    FileType::Unknown => {
                        self.state = LoaderState::Complete;
                    }
                }
            }

            LoaderState::WaitingForBasic => {
                if self.is_basic_ready() {
                    eprintln!("[LOADER] BASIC ready, starting to type");
                    self.state = LoaderState::TypingBasic;
                    // Get current CPU cycles for timing
                    if let Some(cpu) = self.cpu.upgrade() {
                        self.next_char_at = cpu.borrow().cycles();
                    }
                }
            }

            LoaderState::TypingBasic => {
                // Type one character when ready
                if let Some(cpu) = self.cpu.upgrade() {
                    let cycles = cpu.borrow().cycles();
                    if cycles >= self.next_char_at {
                        self.type_next_char();
                        self.next_char_at = cycles + Self::CHAR_DELAY;
                    }
                }
            }

            LoaderState::LoadingPrg => {
                // Not used in current implementation
            }

            LoaderState::Running | LoaderState::Complete => {
                // Nothing to do
            }
        }

        true
    }

    /// Check if loader has finished
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.state == LoaderState::Complete || self.state == LoaderState::Running
    }

    /// Get current state
    #[allow(dead_code)]
    pub fn state(&self) -> LoaderState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_prg_by_extension() {
        let path = Path::new("test.prg");
        let data = vec![0x01, 0x08]; // Load address $0801
        assert_eq!(Loader::detect_type(path, &data), FileType::Prg);
    }

    #[test]
    fn test_detect_bas_by_extension() {
        let path = Path::new("test.bas");
        let data = b"10 PRINT \"HELLO\"".to_vec();
        assert_eq!(Loader::detect_type(path, &data), FileType::Bas);
    }

    #[test]
    fn test_detect_prg_by_content() {
        let path = Path::new("test.bin");
        let data = vec![0x01, 0x08, 0x00, 0x00]; // Load address $0801
        assert_eq!(Loader::detect_type(path, &data), FileType::Prg);
    }

    #[test]
    fn test_detect_bas_by_content() {
        let path = Path::new("test.txt");
        let data = b"10 PRINT \"HELLO\"\n20 GOTO 10".to_vec();
        assert_eq!(Loader::detect_type(path, &data), FileType::Bas);
    }

    #[test]
    fn test_unknown_file_type() {
        let path = Path::new("test.xyz");
        let data = vec![0x00, 0x00, 0xFF, 0xFF]; // Invalid load address
        assert_eq!(Loader::detect_type(path, &data), FileType::Unknown);
    }
}
