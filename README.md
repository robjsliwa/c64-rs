# c64-rs

Commodore 64 emulator written in Rust.

## Usage

```
cargo run --release -- <path to rom>
```

## Installation

```
brew install sdl2
```

Add this line to your ~/.zshenv or ~/.bash_profile depending on whether you use ZSH or Bash.

```
export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"
```
