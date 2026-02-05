# cli-modbus-viewer

CLI utility for polling Modbus TCP devices with a tabular view of registers.

![demo](https://raw.githubusercontent.com/Lolita1001/cli-modbus-viewer/assets/media/cli-modbus-viewer.png)

Russian version: [README.md](README.md)

## Features

- Connect to Modbus TCP devices
- Tabular register display
- Multiple formats: Hex, Int16, UInt16, Binary, Bool
- Supports all register types: Holding, Input, Coils, Discrete
- Watch mode for continuous monitoring

## Installation

```bash
cargo build --release
```

Run after build:

```bash
./target/release/cli-modbus-viewer --help
```

## Usage

`-h/--host` accepts an IP address or hostname (e.g. `localhost`).

```bash
# Basic poll
cli-modbus-viewer -h 192.168.1.100 --holding 0-10

# Default type (holding) if register types are not specified
cli-modbus-viewer -h 192.168.1.100 0-10

# Poll different register types
cli-modbus-viewer -h 192.168.1.100 --holding 0-5 --input 10-15 --coils 0-7

# Port / unit id / timeout
cli-modbus-viewer -h 192.168.1.100 -p 502 -u 1 -t 2000 --holding 0-20

# Watch mode
cli-modbus-viewer -h 192.168.1.100 --holding 0-10 -w --interval 500
```


## Project structure

```
modbus-viewer/
├── Cargo.toml          # Project dependencies
├── Cargo.lock          # Dependency lockfile
├── README.md           # Project description (RU)
├── README.en.md        # Project description (EN)
├── src/
│   ├── main.rs         # Entry point
│   ├── cli.rs          # CLI (clap) and argument validation
│   ├── addr.rs         # Register address parser
│   ├── modbus.rs       # Modbus TCP: connect and read registers
│   └── render.rs       # Table rendering and value formatting
└── target/             # Build artifacts
```

