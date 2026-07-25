# rfce

rfce is a Famicom / NES emulator written in rust.

## Mapper support

See [support.md](support.md)

## Requirements

[SDL3](libsdl.org) is required to run rfce. SDL3 is available through various package managers, such as:

```sh
# debian-based linux system
~ $ sudo apt install libsdl3

# fedora-based linux system
~ $ sudo dnf install sdl3

# arch-based linux system
~ $ sudo pacman -S sdl3

# macOS system
~ $ brew install sdl3

# mingw windows system
~ $ pacman -S sdl3
```

On systems where a SDL3 package is not available (or you simply want to compile SDL3 manually), the `build-sdl3` feature can be enabled.

## Building

```sh
# Build normally
~ $ cargo build --release

# With the `build-sdl3` feature
~ $ cargo build --release -F build-sdl3
```

## Running

To run rfce with a GUI, simply run the command itself.

```sh
# Run normally
~ $ rfce

# Optionally specify a file to load and start running
~ $ rfce <file.nes>

# Run without a GUI (starts a debugger)
~ $ rfce --headless <file.nes>

# Run with additional logging information (see the `env_logger` crate for more info)
~ $ RUST_LOG=info rfce
```

## Useful sources

- The [Nesdev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)
- The [Mesen emulator](https://www.github.com/SourMesen/Mesen2). It's extensive debugging capabilities are especially useful!