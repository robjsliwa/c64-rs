# c64-rs

Commodore 64 emulator written in Rust.

## Current Status

A functional Commodore 64 emulator with working CPU, memory, graphics, audio, and program loading capabilities. The emulator can run BASIC programs and machine language programs with full VIC-II graphics and SID audio support.

### Implemented and Working

- **CPU (MOS 6510)** ✅ Complete
  - All official 6502 opcodes (56 instructions across 151 opcode variants)
  - All 13 addressing modes
  - Correct flag handling (N, V, B, D, I, Z, C)
  - Stack operations
  - Interrupt handling (IRQ, NMI, BRK)
  - JMP indirect page boundary bug emulation
  - Passes Klaus Dormann 6502 functional test suite

- **Memory** ✅ Complete
  - 64KB address space
  - ROM/RAM bank switching
  - Memory-mapped I/O for all chips
  - C64 ROM loading (BASIC, KERNAL, Character ROM)

- **VIC-II (Video Chip)** ✅ Fully Functional
  - All graphics modes:
    - Standard character mode (40x25 text)
    - Multicolor character mode
    - Standard bitmap mode (320x200 hi-res)
    - Multicolor bitmap mode (160x200)
    - Extended background color mode
  - Full sprite support (8 hardware sprites)
  - Sprite priorities and collisions
  - Border and background colors
  - Raster interrupts
  - VIC banking (CIA2 control)
  - Smooth scrolling

- **SID (Sound Chip)** ✅ Fully Functional
  - 3 independent voices with oscillators
  - Multiple waveforms: Triangle, Sawtooth, Pulse, Noise
  - ADSR envelope generators
  - Ring modulation and oscillator sync
  - Resonant multimode filter (low-pass, high-pass, band-pass)
  - Real-time audio output via SDL2

- **CIA Chips** ✅ Working
  - CIA1: Keyboard matrix, timers, interrupts
  - CIA2: VIC bank switching, timers

- **Program Loader** ✅ Fully Functional
  - `.prg` files (binary programs with load address)
    - Supports both BASIC programs ($0801) and machine language
    - Automatic detection and execution
  - `.bas` files (plain text BASIC programs)
    - Automatic typing into BASIC interpreter
    - Auto-run after loading
  - Command-line file loading

- **I/O** ✅ Working
  - SDL2 graphics window (384x272 display)
  - Keyboard input via CIA1 matrix
  - Audio output (44.1kHz, 16-bit)

### Not Yet Implemented

- Tape and disk drive emulation
- Joystick input
- Full timing accuracy for edge cases
- Save state functionality

## Building

### Prerequisites

The emulator requires SDL2 for graphics and audio.

#### macOS

```bash
brew install sdl2
```

Add this to your `~/.zshenv` or `~/.bash_profile`:

```bash
export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"
```

#### Linux (Debian/Ubuntu)

```bash
sudo apt-get install libsdl2-dev
```

#### Windows

Download and install SDL2 development libraries from [libsdl.org](https://www.libsdl.org/), then add them to your PATH.

### Compile

Build the release version (recommended for performance):

```bash
cargo build --release
```

The binary will be in `target/release/c64-rs`.

For development with faster compile times:

```bash
cargo build
```

## Running the Emulator

### Quick Start

Load and run a BASIC program:

```bash
./target/release/c64-rs assets/bas/10print.bas
```

Load and run a machine language program:

```bash
./target/release/c64-rs assets/prg/monopole.prg
```

Run the emulator without a program (starts in BASIC):

```bash
./target/release/c64-rs
```

### Loading Programs

#### .bas Files (BASIC Text Programs)

BASIC programs in plain text format (with `.bas` extension) are automatically:
1. Detected as BASIC source code
2. Typed into the C64 BASIC interpreter character by character
3. Run automatically with the `RUN` command

Example:
```bash
./target/release/c64-rs assets/bas/bouncing.bas
```

The loader waits for BASIC to be ready, types the program, and executes it.

#### .prg Files (Binary Programs)

PRG files contain binary data with a 2-byte load address header. The emulator:
1. Reads the load address from the first 2 bytes (little-endian)
2. Loads the program data into memory at that address
3. For BASIC programs (load address $0801), integrates with BASIC and runs
4. For machine language programs (other addresses), jumps directly to the load address

Example:
```bash
./target/release/c64-rs assets/prg/monopole.prg
```

Common PRG load addresses:
- `$0801` (2049) - BASIC programs
- `$1000` (4096) - Machine language programs
- `$C000` (49152) - ML programs in high memory

### Available Example Programs

**BASIC Programs** (`assets/bas/`):
- `10print.bas` - Classic maze generator
- `bouncing.bas` - Bouncing ball animation
- `bitmap.bas` - Bitmap graphics demo
- `lowercase.bas` - Lowercase/uppercase text demo
- `simple_tone.bas` - Simple SID audio test
- `sound_test.bas` - Multi-voice SID demo
- `test_chars.bas` - Character set display
- `test_simple.bas` - Basic functionality test

**Machine Language Programs** (`assets/prg/`):
- `monopole.prg` - Audio/visual demo

### Testing and Debugging

#### Run CPU Functional Test

Verify the CPU passes the Klaus Dormann 6502 functional test:

```bash
./target/release/c64-rs test
```

Expected output:
```
Test mode enabled
Test PASSED after 26764663 cycles!
```

#### Debug Mode

Interactive debugger for stepping through code:

```bash
./target/release/c64-rs debug
```

Commands:
- `step` - Execute one CPU instruction
- `load` - Load a byte into memory
- `display` - Display memory contents
- `quit` - Exit debugger

## Requirements

### C64 ROM Files

The emulator requires original C64 ROM files to run. Place these files in `assets/roms/`:

- `basic.901226-01.bin` - BASIC ROM (8KB)
- `kernal.901227-03.bin` - KERNAL ROM (8KB)
- `characters.901225-01.bin` - Character ROM (4KB)

These ROM files are copyrighted and not included in this repository. You must obtain them legally from your own C64 or from authorized sources.

Without ROMs, the emulator can still:
- Run the CPU functional test (`test` command)
- Use debug mode
- Load and execute some machine language programs directly

## Project Structure

```
c64-rs/
├── src/
│   ├── main.rs           # Entry point, CLI, main loop
│   ├── cpu.rs            # MOS 6510 CPU emulation
│   ├── memory.rs         # Memory management, bank switching
│   ├── cia1.rs           # CIA1 (keyboard, timers, interrupts)
│   ├── cia2.rs           # CIA2 (VIC banking, serial port)
│   ├── io.rs             # SDL2 I/O layer (graphics, keyboard, audio)
│   ├── audio.rs          # Audio callback and buffer management
│   ├── loader/           # Program loader
│   │   ├── mod.rs        # PRG/BAS file loading and auto-run
│   │   └── README.md     # Loader implementation details
│   ├── vic/              # VIC-II video chip
│   │   ├── mod.rs        # Main VIC implementation
│   │   ├── registers.rs  # Register definitions
│   │   ├── colors.rs     # C64 color palette
│   │   ├── sprites.rs    # Sprite rendering
│   │   └── constants.rs  # Timing and layout constants
│   ├── sid/              # SID sound chip
│   │   ├── mod.rs        # Main SID controller
│   │   ├── voice.rs      # Voice state machine
│   │   ├── oscillator.rs # Waveform generation
│   │   ├── envelope.rs   # ADSR envelope
│   │   ├── filter.rs     # Resonant filter
│   │   └── README.md     # SID implementation details
│   └── cia/              # CIA chip common code
│       ├── mod.rs
│       ├── base.rs       # CIA base functionality
│       └── timer.rs      # Timer implementation
├── assets/
│   ├── roms/             # C64 ROM files (not included)
│   ├── bas/              # BASIC program examples
│   ├── prg/              # Machine language programs
│   └── tests/            # Test ROMs
├── tests/                # Rust integration tests
│   ├── loader_tests.rs
│   ├── loader_char_tests.rs
│   └── loader_10print_test.rs
├── c64rs-design/         # Design documents and specifications
└── Cargo.toml            # Rust project configuration
```

## Technical Details

### Architecture

- **CPU Emulation**: Cycle-accurate MOS 6510 with full instruction set
- **Graphics**: Scanline-based VIC-II rendering at 320x200 (or 384x272 with borders)
- **Audio**: Real-time SID synthesis at 44.1kHz sample rate
- **Timing**: Approximately 1MHz CPU clock with proper cycle counting
- **Memory**: Full 64KB address space with ROM/RAM banking

### Performance

The emulator runs at native C64 speed on modern hardware. The release build is highly optimized and can handle real-time audio and graphics without frame drops on most systems.

## Development

### Running Tests

```bash
cargo test
```

### Documentation

Design documentation and implementation notes are in the `c64rs-design/` directory:
- `architecture.md` - Overall system architecture
- `phases.md` - Development phases and roadmap
- `stories/` - Detailed implementation stories for each phase
- `SID_IMPLEMENTATION.md` - SID chip details
- `LOADER_IMPLEMENTATION.md` - Program loader details

## Known Issues

- Some timing-sensitive programs may not work perfectly
- Cycle accuracy is approximate, not exact
- No disk drive or tape emulation yet

## References

- [Klaus Dormann 6502 Test Suite](https://github.com/Klaus2m5/6502_65C02_functional_tests)
- [C64 Wiki](https://www.c64-wiki.com/)
- [6502.org](http://www.6502.org/)

## License

MIT
