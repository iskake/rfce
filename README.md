# rfce

rfce is a Famicom / NES emulator written in Rust.

![Screenshot of rfce running and displaying the title screen of Kirby's Adventure (JP version)](img/ka.png)

## Requirements

[SDL3](libsdl.org) is required to run. SDL3 is available through various package managers, such as:

```sh
# debian-based linux system
~ $ sudo apt install libsdl3

# fedora-based linux system
~ $ sudo dnf install sdl3

# arch-based linux system
~ $ sudo pacman -S sdl3

# macOS system
~ $ brew install sdl3
```

On systems where a SDL3 package is not available (or you simply want to compile SDL3 manually), either of the `build-sdl3` or `build-sdl3-static` (for dynamic/static linking respectively) features can be enabled (both require [CMake](https://cmake.org/).)

## Building

rfce can be easily built using [cargo](https://rust-lang.org/tools/install/):

```sh
# Build normally
~ $ cargo build --release

# With the `build-sdl3` feature
~ $ cargo build --release -F build-sdl3
```

## Running

```sh
# Run normally
~ $ rfce

# Optionally specify a file to load and start running
~ $ rfce <file.nes>

# Run without a GUI (starts a debugger)
~ $ rfce --headless <file.nes>
```

## Emulator status

### Mapper support

rfce has support for the following mappers:

- NROM
- UxROM
- MMC1
- MMC2
- MMC3

These mappers (especially MMC1 and MMC3) account for a large number of first-party games.

(Note that not all games utilizing these mappers have been tested (both MMC1 and MMC3 are used in 300+ games), so your mileage may vary.)

### Missing features of note

The following is an incomplete list of features that are not (yet) implemented.

- Audio (APU & SDL audio output)
- Famicom Disk System emulation
- Any and all other mappers
- PAL game support (games _may_ still run, but are likely going to be faster than normal due to running at ~60hz instead of the usual ~50hz)

## Useful sources

- [Nesdev.org](https://www.nesdev.org/) (especially the [Nesdev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)) contains essentially everything there is to know about the Famicom and NES.
- The [Mesen emulator](https://www.github.com/nesdev-org/MesenCE). It's high accuracy and extensive debugging capabilities are especially useful!