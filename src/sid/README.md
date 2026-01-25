# SID (Sound Interface Device) Implementation

## Overview

This implementation provides full emulation of the C64's SID chip (6581/8580), including:

- 3 independent voices with oscillators and envelopes
- Multiple waveforms: Triangle, Sawtooth, Pulse, and Noise
- ADSR envelope generators
- Ring modulation and hard sync
- Multi-mode filter (low-pass, band-pass, high-pass, and combinations)
- Master volume control

## Architecture

### Module Structure

```
src/sid/
├── mod.rs          - Main SID chip implementation
├── voice.rs        - Voice with oscillator and envelope
├── oscillator.rs   - Waveform generation
├── envelope.rs     - ADSR envelope generator
└── filter.rs       - State-variable filter
```

### Memory Map

The SID occupies memory addresses $D400-$D41C (29 registers):

#### Voice 1 ($D400-$D406)
- $D400-$D401: Frequency (16-bit)
- $D402-$D403: Pulse Width (12-bit)
- $D404: Control Register (waveform, gate, sync, ring mod, test)
- $D405: Attack/Decay
- $D406: Sustain/Release

#### Voice 2 ($D407-$D40D)
Same layout as Voice 1

#### Voice 3 ($D40E-$D414)
Same layout as Voice 1

#### Filter & Volume ($D415-$D418)
- $D415-$D416: Filter Cutoff (11-bit)
- $D417: Filter Resonance & Routing
- $D418: Filter Mode & Master Volume

#### Read-Only Registers
- $D41B: OSC3 - Voice 3 Oscillator Output
- $D41C: ENV3 - Voice 3 Envelope Output

## Usage Example

```basic
10 REM SIMPLE SOUND TEST
20 POKE 54296,15        :REM VOLUME MAX
30 POKE 54277,9         :REM ATTACK/DECAY
40 POKE 54278,0         :REM SUSTAIN/RELEASE
50 POKE 54273,28        :REM FREQUENCY HI
60 POKE 54272,214       :REM FREQUENCY LO
70 POKE 54276,17        :REM GATE ON, TRIANGLE
80 FOR I=1 TO 500:NEXT  :REM WAIT
90 POKE 54276,16        :REM GATE OFF
100 END
```

## Implementation Details

### Oscillators

Each voice has a 24-bit phase accumulator that increments by the frequency value each clock cycle (~1MHz). The upper 12 bits are used to generate waveforms:

- **Triangle**: Uses bit 11 to determine rising/falling edge
- **Sawtooth**: Direct output of upper 12 bits
- **Pulse**: Compares phase to pulse width
- **Noise**: 23-bit LFSR (Linear Feedback Shift Register)

### ADSR Envelope

The envelope generator has four phases:

1. **Attack**: Linear rise to maximum (255)
2. **Decay**: Exponential fall to sustain level
3. **Sustain**: Hold at sustain level
4. **Release**: Exponential fall to zero

Rate tables determine the speed of each phase, matching the original SID timing.

### Filter

Implements a state-variable filter with:
- Cutoff frequency (11-bit, non-linear)
- Resonance (4-bit)
- Multiple modes: LP, BP, HP, and combinations
- Per-voice filter routing

### Special Features

#### Ring Modulation
- Voice 1 ring mods with Voice 3
- Voice 2 ring mods with Voice 1  
- Voice 3 ring mods with Voice 2
- Only affects triangle waveform
- XORs oscillator MSB with output

#### Hard Sync
- Voice 1 synced by Voice 3
- Voice 2 synced by Voice 1
- Voice 3 synced by Voice 2
- Resets phase on source zero-crossing

## Integration

The SID is integrated into the main emulation loop:

1. Registered with Memory for I/O routing at $D400-$D4FF
2. Clocked every CPU cycle to advance oscillators and envelopes
3. Samples generated at audio rate (44.1kHz) based on cycle accumulation

## Testing

Comprehensive unit tests cover:
- All waveform generators
- ADSR envelope phases
- Filter modes
- Register read/write
- Voice control

Run tests with: `cargo test sid`

## Future Enhancements

Potential improvements for full accuracy:

1. Audio output via SDL (currently prepared but not connected)
2. More accurate filter frequency response curve
3. Combined waveforms (multiple waveforms selected simultaneously)
4. Waveform/envelope interaction artifacts
5. External audio input processing
