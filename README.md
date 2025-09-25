# terminal_analog_clock

A simple, customizable analog clock that runs directly in your terminal.

## Features

* Real-time analog display with hour, minute, and second hands.

* Customizable appearance with interactive controls.

* Smooth "sweep" second hand movement for modern terminals.

* Lightweight and dependency-free.

## Installation

1. **Prerequisites**: Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

2. **Build**: Clone the repository and run the following command from the project's root directory:

```
cargo build --release
```
or just type ```make``` if you have [make](https://www.gnu.org/software/make/) installed.

This creates the ```target/release/tag``` executable you can put anywhere.

# Controls

The clock's appearance can be changed in real-time using the following keys:

| Key | Action | 
| ----- | ----- | 
| `s` | **Toggle Second Hand**: Cycles through three modes: - Off (hidden) - Tick (updates every second) - Sweep (continuous movement) | 
| `c` | **Toggle Clock Face**: Cycles through four styles: - Full circle outline - Minute and hour ticks - Hour ticks only - Blank | 
| `n` | **Toggle Hour Markers**: Cycles through three styles: - Off (no markers) - Numeric (12, 3, 6, 9) - Dots | 
| `+` | Increases the clock's width (makes it wider). | 
| `-` | Decreases the clock's width (makes it narrower). | 
| `q` | Quits the application. | 

