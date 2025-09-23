# Chip-8 Emulator

A modern Chip-8 emulator written in Rust with a focus on accuracy, clean architecture, and comprehensive testing.

## Features

### ✨ Core Emulation

- **Complete Instruction Set**: All 36 standard Chip-8 instructions fully implemented
- **Cycle-Accurate Timing**: 60Hz timer system with configurable CPU speed (100-2000 Hz)
- **Memory Management**: Full 4KB memory space with proper font data and ROM loading
- **Hardware Abstraction**: Clean trait-based separation between emulator core and frontend
- **Compatibility Modes**: Classic CHIP-8 behavior with memory wraparound support

### 🎮 Interfaces

- **GUI Mode**: Basic graphical interface with window-based rendering (default)
- **CLI Tools**: ROM information and validation subcommands
- **Hardware Abstraction**: Display, Audio, and Input traits for flexible frontends

### 🔊 Audio System

- **Real-time Audio**: CPAL-based audio output with configurable frequency
- **Multiple Waveforms**: Sine, square, sawtooth, and triangle wave generation
- **Buzzer Implementation**: Classic Chip-8 beep sound on timer activation

### 🎨 Graphics System

- **64×32 Display**: Standard Chip-8 resolution with collision detection
- **Sprite Rendering**: XOR-based pixel drawing with proper coordinate wrapping
- **Window Scaling**: Automatic window sizing for visibility

### ⌨️ Input System

- **16-Key Hexadecimal Keypad**: Full 0-F key emulation
- **QWERTY Mapping**: Intuitive keyboard layout

  ```
  Chip-8 Keypad:    QWERTY Keyboard:
  1 2 3 C           1 2 3 4
  4 5 6 D           Q W E R
  7 8 9 E           A S D F
  A 0 B F           Z X C V
  ```

### 🔧 Compatibility Features

- **Classic Mode**: Original CHIP-8 behavior (500 Hz, memory wraparound enabled)
- **Modern Mode**: Strict bounds checking and higher speed (700 Hz default)
- **Memory Wraparound**: Configurable out-of-bounds memory access behavior
- **CPU Speed Control**: Adjustable instruction execution rate

## Installation

### Prerequisites

- **Rust**: 1.70+ (2021 edition)
- **System Dependencies**:
  - **Linux**: `libasound2-dev`, `libxcb-dev`
  - **macOS**: No additional dependencies (built-in support)
  - **Windows**: No additional dependencies (built-in support)

### Quick Install

```bash
# Clone the repository
git clone https://github.com/bclews/chip8.git
cd chip8

# Build and install the binary to ~/.cargo/bin
make install

# Verify installation
chip8 --version
```

### Manual Build

```bash
# Development build
cargo build

# optimised release build
cargo build --release

# Run directly without installing
cargo run -- roms/pong.ch8
```

## Usage

### GUI Mode (Default)

The emulator launches a graphical window when you provide a ROM file:

```bash
# Launch GUI with a ROM file
chip8 roms/pong.ch8

# Or via cargo
cargo run -- roms/pong.ch8

# Or via Makefile
make run ROM=roms/pong.ch8
```

**GUI Configuration:**

- The GUI respects `--config` and `--profile` flags for full customization
- Without configuration, uses sensible defaults: classic green-on-black display, 440Hz square wave at 30% volume
- Individual CLI flags (like `--scale` or `--volume`) are not available; use `--config` or `--profile` instead

### CLI Subcommands

#### Show ROM Information

```bash
# Display ROM file metadata
chip8 info roms/game.ch8

# Example output:
# ROM Information:
# File: roms/game.ch8
# Size: 246 bytes
# Max size: 3584 bytes
# ✅ Size is valid
# ✅ ROM contains data
#
# First 16 bytes:
# 0200: 00 E0 A2 2A 60 0C 61 08 ...
```

#### Validate ROM File

```bash
# Validate ROM file format
chip8 validate roms/game.ch8

# Example output:
# Validating ROM: roms/game.ch8
# ✅ ROM validation passed
```

### Command-Line Flags

| Flag        | Short | Description                                                            |
| ----------- | ----- | ---------------------------------------------------------------------- |
| `--config`  |       | Load configuration from TOML file                                      |
| `--profile` |       | Use configuration preset (classic, modern, gaming, development, retro) |
| `--verbose` |       | Enable verbose logging                                                 |
| `--help`    | `-h`  | Show help information                                                  |
| `--version` | `-V`  | Show version information                                               |

### Examples

```bash
# Get help
chip8 --help

# Show version
chip8 --version

# Analyze a ROM
chip8 info roms/space_invaders.ch8

# Validate before running
chip8 validate roms/test.ch8 && chip8 roms/test.ch8

# Run with GUI (default behavior)
chip8 roms/pong.ch8

# Run with a configuration profile
chip8 --profile classic roms/pong.ch8

# Run with a custom configuration file
chip8 --config my-config.toml roms/pong.ch8
```

## Configuration (Advanced)

The emulator has a sophisticated configuration system with presets for different use cases. You can configure the emulator via TOML files (`--config` flag) or by selecting a configuration profile (`--profile` flag).

### Configuration Profiles

The emulator includes 5 configuration presets that can be selected with the `--profile` flag:

- **classic** - Original CHIP-8 behavior (500 Hz, memory wraparound enabled, green-on-black display)
- **modern** - Default mode (700 Hz, strict bounds checking, high contrast display)
- **gaming** - optimised for gameplay (700 Hz, responsive audio, no smooth scaling)
- **development** - Debug-friendly settings (slower execution, verbose logging enabled)
- **retro** - Nostalgic amber-on-black display with classic timing

Example: `chip8 --profile gaming roms/pong.ch8`

### Custom Configuration Files

You can create your own TOML configuration files to customize all aspects of the emulator. Example configuration files are provided in the `examples/` directory:

- **examples/example-config.toml** - Comprehensive example with all options documented
- **examples/config-retro.toml** - Amber CRT monitor aesthetic with classic timing
- **examples/config-gaming.toml** - optimised for modern gameplay with high contrast

Example usage:

```bash
chip8 --config examples/config-retro.toml roms/pong.ch8
```

### Programmatic Configuration

When using the emulator as a Rust library, you can also configure it programmatically:

```rust
use chip8::frontend::EmulatorConfig;

// Classic CHIP-8 behavior (500 Hz, wraparound enabled)
let config = EmulatorConfig::classic();

// Modern interpretation (700 Hz, strict bounds)
let config = EmulatorConfig::modern();

// Gaming optimised (fast, good audio)
let config = EmulatorConfig::gaming();

// Development (slower, debug-friendly)
let config = EmulatorConfig::development();
```

### Configuration Options

The following configuration options exist in the codebase:

```toml
[behavior]
cpu_speed = 700              # Instructions per second (100-2000)
memory_wraparound = false    # Enable memory wraparound
strict_bounds = true         # Strict bounds checking
timer_frequency = 60         # Timer update frequency in Hz

[graphics]
scale_factor = 10                                          # Pixel scale factor (1-20)
foreground_color = { r = 0, g = 255, b = 0, a = 255 }     # Foreground color (RGBA)
background_color = { r = 0, g = 0, b = 0, a = 255 }       # Background color (RGBA)
smooth_scaling = true                                      # Enable smooth scaling
maintain_aspect_ratio = true                               # Maintain 2:1 aspect ratio

[audio]
frequency = 440              # Buzzer frequency in Hz
volume = 0.3                 # Volume level (0.0-1.0)
waveform = "Square"          # Square, Sine, Sawtooth, Triangle
sample_rate = 44100          # Audio sample rate
buffer_size = 512            # Audio buffer size

[keyboard]
capture_enabled = true       # Enable keyboard input capture
polling_rate = 60            # Keyboard polling rate in Hz (1-1000)
handle_repeats = true        # Handle key repeat events
repeat_delay = 250           # Key repeat delay in milliseconds
use_raw_input = false        # Use raw keyboard input (bypasses OS key repeat)

[debug]
enabled = false              # Enable debug mode
break_on_error = false       # Break execution on errors
log_instructions = false     # Log each CPU instruction (very verbose)
```

**Note**: These TOML configuration options can be used by creating a configuration file and loading it with `--config path/to/config.toml`, or by using one of the built-in profiles with `--profile <name>`.

## Development

### Build Commands

```bash
# Development build
make build

# optimised release build
make release

# Run tests
make test

# Run tests with detailed output
make test-verbose

# Check code without building
make check
```

### Code Quality

```bash
# Format code
make fmt

# Check formatting
make fmt-check

# Run linter
make clippy

# Run all quality checks
make lint

# Pre-commit checks (format + lint + test)
make pre-commit
```

### Testing

The emulator has comprehensive test coverage:

```bash
# Run all tests (219 total: 210 unit + 9 integration)
make test

# Run only unit tests
make test-unit

# Run with detailed output
make test-verbose
```

**Test Suite:**

- **210 Unit Tests**: Component-level testing
  - CPU instruction set (arithmetic, logic, jumps, memory)
  - Timer system
  - Memory management
  - Audio system (buzzer, waveforms, configuration)
  - Graphics rendering
  - Input handling
- **9 Integration Tests**: Full system testing
  - ROM loading and execution
  - Hardware system integration
  - Error handling
  - Display sprite operations

### Documentation

```bash
# Generate and open documentation
make doc-open

# Browse the API documentation in your browser
```

## Architecture

### Core Components

- **Emulator Core** (`src/emulator/`)
  - CPU with full instruction set implementation
  - Memory management with wraparound support
  - Register file (16 general purpose + I, PC, SP)
  - Timers (delay and sound) at 60Hz
  - Stack for subroutine calls

- **Hardware Abstraction** (`src/hardware/`)
  - `Display` trait for graphics output
  - `Audio` trait for sound generation
  - `Input` trait for keyboard handling

- **Graphics System** (`src/graphics/`)
  - 64×32 pixel buffer management
  - Sprite rendering with XOR logic
  - Collision detection
  - Color configuration support

- **Audio System** (`src/audio/`)
  - CPAL-based audio streaming
  - Multiple waveform generators
  - Buzzer with configurable frequency/volume
  - Low-latency audio buffer management

- **Input System** (`src/input/`)
  - Keyboard mapper (QWERTY → Chip-8 keypad)
  - Software input implementation
  - Key state tracking

- **Frontend** (`src/frontend/`)
  - CLI argument parsing (clap)
  - GUI implementation (winit + pixels)
  - Configuration management
  - ROM file utilities

### Memory Layout

```
0x000-0x1FF: Interpreter (font data at 0x50-0x9F)
0x200-0xE9F: Program ROM (3584 bytes max)
0xEA0-0xEFF: Call stack / internal use
0xF00-0xFFF: Display refresh / internal use
```

### Design Patterns

- **Hardware Abstraction Layer**: All hardware interactions use trait interfaces
- **Zero-Cost Abstractions**: Generic traits with monomorphization
- **Error Handling**: Structured errors via `thiserror` and `color-eyre`
- **Modular Architecture**: Clean separation between emulation and frontend

## Performance

The emulator is designed for accurate cycle timing while maintaining performance:

- **Target**: 500-700 instructions per second (configurable)
- **Timer Accuracy**: 60Hz update rate
- **Audio Latency**: Low-latency streaming with 512-sample buffers
- **Memory Safety**: Zero unsafe code in core emulation logic

### Optimisation Features

1. **Release Mode optimisation**

   ```bash
   # Build with full optimisations
   cargo build --release
   ```

   - Compiler optimisations enabled
   - Debug assertions removed
   - Conditional logging optimised away
   - Inline critical functions

2. **Memory Access optimisation**
   - Fast-path memory operations
   - Bounds checking optimised in release mode
   - Efficient wraparound using modulo operations

3. **Register Access optimisation**
   - Direct array indexing
   - optimised flag register operations

## ROM Compatibility

The emulator supports standard Chip-8 ROMs with the following specifications:

- **Maximum ROM Size**: 3584 bytes
- **Load Address**: 0x200
- **Instruction Set**: All 36 standard Chip-8 instructions
- **Display**: 64×32 monochrome
- **Sound**: Single beep tone
- **Input**: 16-key hexadecimal keypad

### Classic Mode vs Modern Mode

**Classic Mode** (`--profile classic`):

- CPU Speed: 500 Hz (slower, more authentic)
- Memory Wraparound: Enabled (addresses wrap at 4KB boundary)
- Strict Bounds: Disabled (permits out-of-bounds access)

**Modern Mode** (default, or `--profile modern`):

- CPU Speed: 700 Hz (faster gameplay)
- Memory Wraparound: Disabled (errors on out-of-bounds)
- Strict Bounds: Enabled (catches programming errors)

## Technical Details

### Dependencies

**Core Dependencies:**

- `winit` - Cross-platform window creation
- `pixels` - GPU-accelerated pixel buffer rendering
- `cpal` - Cross-platform audio library
- `rodio` - Audio playback

**Configuration:**

- `clap` - Command-line argument parsing
- `serde` / `toml` - Configuration serialization

**Error Handling:**

- `thiserror` - Error derive macros
- `color-eyre` - Pretty error reports

**Development:**

- `env_logger` - Logging framework
- `log` - Logging facade

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Resources

### Chip-8 Documentation

- [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Wikipedia: CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
- [Mastering CHIP-8](http://mattmik.com/files/chip8/mastering/chip8.html)

### Test ROMs

- Place your Chip-8 ROMs in the `roms/` directory
- Test ROMs can be found online (search for "chip8 test rom")
- Classic games: Pong, Space Invaders, Tetris, etc.
