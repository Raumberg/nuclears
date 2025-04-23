# Nuclear Monitor

A fun CLI-based system monitor that visualizes your system load as a nuclear reactor. The more CPU usage, the more unstable and radioactive the reactor becomes!

## Features

- Real-time CPU and memory usage monitoring
- Animated nuclear reactor visualization
- Control rod simulation based on system load
- Radiation particle effects
- Temperature history graph
- Reactor stability indicators

## Controls

- `q` - Quit the application
- `p` - Pause/Resume monitoring
- `h` - Toggle help screen

## Installation

Ensure you have Rust and Cargo installed. Then clone this repository and build:

```bash
git clone https://github.com/yourusername/nuclear-monitor.git
cd nuclear-monitor
cargo build --release
```

The compiled binary will be in `target/release/nuclear-monitor`.

## Usage

Simply run the executable:

```bash
./target/release/nuclear-monitor
```

## How it works

The application uses:
- `sysinfo` to collect system metrics
- `ratatui` for the terminal UI
- `crossterm` for terminal control
- `rand` for randomized particle effects

The nuclear reactor's stability is directly tied to your CPU usage - higher CPU load makes the reactor more unstable!

## License

MIT 