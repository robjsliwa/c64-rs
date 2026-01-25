# C64 Loader Implementation

This document describes the C64 program loader implementation in the emulator.

## Overview

The loader module enables the emulator to load and run external programs in two formats:
- **PRG files**: Binary programs with a load address header
- **BAS files**: Plain text BASIC programs

## Features

### File Type Detection
- Automatic detection by file extension (.prg, .bas)
- Fallback content-based detection for files without extensions
- Case-insensitive extension matching

### PRG File Loading
PRG files use the standard Commodore 64 format:
```
Byte 0-1: Load address (little-endian)
Byte 2+:  Program data
```

Common load addresses:
- `$0801`: BASIC program start
- `$1000`: Machine language program
- `$C000+`: Upper RAM machine language

When loading PRG files:
1. Load address is read from first 2 bytes
2. Program data is copied to memory starting at load address
3. For BASIC programs ($0801), BASIC pointers are set up
4. For ML programs, the CPU PC is set to the load address

### BAS File Loading
BAS files contain plain ASCII text BASIC programs. The loader:
1. Reads the file as text
2. Converts to uppercase (BASIC standard)
3. Types each character into the emulator via keyboard simulation
4. Adds RETURN after each line
5. Automatically types "RUN" after loading

### State Machine
The loader uses a state machine:
- **Idle**: Initial state, starts loading
- **WaitingForBasic**: Waits for BASIC to be ready (PC at $A65C)
- **TypingBasic**: Types BAS file characters one by one
- **LoadingPrg**: PRG file being loaded (currently unused)
- **Running**: Program loaded and running
- **Complete**: Loading finished

## Usage

### Command Line
```bash
# Load and run a PRG file
./c64-rs program.prg

# Load and run a BAS file
./c64-rs program.bas

# Run without loading a program
./c64-rs
```

### Examples
```bash
# Load the 10print demo
./c64-rs assets/bas/10print.bas

# Load the Monopole demo
./c64-rs assets/prg/monopole.prg

# Load a simple tone test
./c64-rs assets/bas/simple_tone.bas
```

## Implementation Details

### Module Structure
- `src/loader/mod.rs`: Main loader implementation

### Key Components

#### Loader Struct
```rust
pub struct Loader {
    state: LoaderState,
    file_type: FileType,
    data: Vec<u8>,
    position: usize,
    char_queue: VecDeque<char>,
    memory: Weak<RefCell<Memory>>,
    cpu: Weak<RefCell<Cpu>>,
    keyboard: Weak<RefCell<IO>>,
    next_char_at: u32,
}
```

#### State Enums
```rust
pub enum LoaderState {
    Idle,
    WaitingForBasic,
    TypingBasic,
    LoadingPrg,
    Running,
    Complete,
}

pub enum FileType {
    Prg,
    Bas,
    Unknown,
}
```

### Integration with Main Loop
The loader's `emulate()` method is called each frame in the main emulation loop, allowing it to:
- Progress through loading states
- Type characters at the appropriate rate
- Detect when BASIC is ready

### Timing
- Characters are typed at intervals controlled by `WAIT_DURATION` (18000 cycles)
- This ensures the keyboard buffer doesn't overflow
- BASIC detection checks if CPU PC is at $A65C (BASIC ready loop)

## Testing

### Unit Tests
The module includes unit tests for:
- File type detection by extension
- File type detection by content
- Unknown file type handling

Run tests with:
```bash
cargo test loader::
```

### Integration Tests
Integration tests verify:
- PRG files can be loaded
- BAS files are readable
- All example programs are accessible

Run integration tests with:
```bash
cargo test loader_integration_tests
```

## Error Handling

The loader handles various error conditions:
- `IoError`: File not found or read error
- `InvalidProgram`: Invalid PRG format
- `NoMemory`: Memory reference not available
- `UnknownFileType`: File type cannot be determined

## Future Enhancements

Possible future improvements:
- Drag-and-drop support for loading files
- T64 tape archive support
- D64 disk image support
- Snapshot save/load
- Pause/resume during typing
- Configurable typing speed

## References

- [Phase 7 Design Document](../c64rs-design/stories/phase7-loader.md)
- [C64 PRG Format](https://vice-emu.sourceforge.io/vice_16.html#SEC352)
- [C64 Memory Map](https://www.c64-wiki.com/wiki/Memory_Map)
