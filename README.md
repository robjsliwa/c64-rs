# c64-rs

Commodore 64 emulator written in Rust.

## Current Status

**Phase 1 (CPU) - Complete**

The MOS 6510 CPU emulation is fully functional and passes the Klaus Dormann 6502 functional test suite.

### Implemented Features

- **CPU (MOS 6510)**
  - All official 6502 opcodes (56 instructions across 151 opcode variants)
  - All 13 addressing modes (Immediate, Zero Page, Zero Page X/Y, Absolute, Absolute X/Y, Indirect, Indexed Indirect, Indirect Indexed, Implied, Accumulator, Relative)
  - Correct flag handling (N, V, B, D, I, Z, C)
  - Stack operations
  - Interrupt handling (IRQ, NMI, BRK)
  - JMP indirect page boundary bug emulation

- **Memory**
  - 64KB address space
  - ROM/RAM bank switching
  - Memory-mapped I/O infrastructure

- **CIA Chips (Partial)**
  - CIA1 and CIA2 timer infrastructure
  - Keyboard matrix support (CIA1)

- **I/O**
  - SDL2 window initialization
  - Keyboard event handling infrastructure

### Not Yet Implemented

- VIC-II (video chip) - graphics rendering
- SID (sound chip) - audio
- Full C64 boot sequence with BASIC/KERNAL ROMs
- Program loading (.prg, .bas files)

## Building

### Prerequisites

#### macOS

```bash
brew install sdl2
```

Add this line to your `~/.zshenv` or `~/.bash_profile`:

```bash
export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"
```

#### Linux (Debian/Ubuntu)

```bash
sudo apt-get install libsdl2-dev
```

#### Windows

Install SDL2 development libraries and ensure they're in your PATH.

### Compile

```bash
cargo build --release
```

## Usage

### Run CPU Functional Test

Verify the CPU implementation passes the Klaus Dormann 6502 functional test:

```bash
cargo run --release -- test
```

Expected output:
```
Test mode enabled
Test PASSED after 26764663 cycles!
```

### Debug Mode

Interactive debugger for stepping through code:

```bash
cargo run --release -- debug
```

Commands:
- `step` - Execute one instruction
- `load` - Load a byte into memory
- `display` - Display memory contents
- `quit` - Exit debugger

### Run Emulator (WIP)

```bash
cargo run --release
```

Note: Full emulation requires C64 ROM files in `assets/roms/`:
- `basic.901226-01.bin` - BASIC ROM
- `kernal.901227-03.bin` - KERNAL ROM
- `characters.901225-01.bin` - Character ROM

## Project Structure

```
c64-rs/
├── src/
│   ├── main.rs      # Entry point, CLI handling
│   ├── cpu.rs       # MOS 6510 CPU emulation
│   ├── memory.rs    # Memory management, bank switching
│   ├── cia1.rs      # CIA1 (keyboard, timers)
│   ├── cia2.rs      # CIA2 (VIC banking, serial)
│   ├── io.rs        # SDL2 I/O layer
│   └── common.rs    # Shared types
├── assets/
│   ├── roms/        # C64 ROM files (not included)
│   ├── tests/       # Test ROMs
│   ├── bas/         # BASIC programs
│   └── prg/         # Machine code programs
└── c64rs-design/    # Design documentation
```

## Testing

The emulator uses the Klaus Dormann 6502 functional test suite to verify CPU correctness:

- **Test binary**: `assets/tests/6502_functional_test.bin`
- **Test source**: `assets/tests/6502_functional_test.a65`
- **Listing file**: `assets/tests/6502_functional_test.lst` (for debugging)

## References

- [Klaus Dormann 6502 Test Suite](https://github.com/Klaus2m5/6502_65C02_functional_tests)
- [C64 Wiki](https://www.c64-wiki.com/)
- [6502.org](http://www.6502.org/)

## License

MIT
