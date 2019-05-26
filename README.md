# Rusty Chip-8
## NOTE: This library is currently still in development, some features and documentation are still lacking.

Rusty Chip-8 is a chip-8 interpreter library. It is intended to be a highly flexible library for creation of Chip-8 emulators. It provides a simple-to-use and accurate Chip-8 interpreter with simple functions for interacting with the system. All other features (GUI, controlled emulation speed, loading programs from files) are up the use user do create.
## Features
 - All Opcodes for the original Chip-8 system implemented.
 - The Chip-8 Keyboard, screen, and delay/buzzer counters have been implemented.
 - Easy-to-use traits for interacting with the system.
## Usage
Add this to your project's cargo.toml file:
```toml
[dependencies]
rusty-chip8 = { git = "https://github.com/KaComet/rusty-chip8" }
```
## Planned Features
 - A Super Chip-8 interpreter
 - Automatic tests for all functions

