# ggwave-rs

Rust bindings and a small safe wrapper for
[ggwave](https://github.com/ggerganov/ggwave), a library that encodes short
payloads into audio waveforms.

## Features
- Safe `GgWave` wrapper for init/encode/decode
- Raw C FFI bindings available under `ggwave_rs::ffi`
- Build with a vendored upstream copy (default) or a system `libggwave`
- CLI tool with WAV file support for encode/decode

## Requirements
- Rust toolchain (edition 2021)
- Vendored build: a C++11 toolchain
- System build: `libggwave` + `pkg-config` available on your system

## Getting the sources
This repo tracks ggwave as a submodule under `vendor/ggwave`:

```sh
git submodule update --init --recursive
```

## Build
This is a Cargo workspace with two crates:
- `ggwave-rs` — the library
- `ggwave-cli` — the command-line tool

Build everything (vendored, default):

```sh
cargo build
```

Build with system library:

```sh
cargo build --no-default-features --features system
```

Note: the system `libggwave` must be built with the full protocol set
(i.e. without `GGWAVE_CONFIG_FEW_PROTOCOLS` / Arduino configs).

## CLI usage
The CLI reads and writes standard WAV files:

```sh
# Build and install the CLI
cargo install --path ggwave-cli

# Encode a message to a WAV file
ggwave encode "hello" output.wav
ggwave encode "hello" output.wav --volume 30 --protocol ultrasound-fast

# Decode a message from a WAV file
ggwave decode output.wav
```

Available protocols: `audible-normal`, `audible-fast`, `audible-fastest`,
`ultrasound-normal`, `ultrasound-fast`, `ultrasound-fastest`,
`dt-normal`, `dt-fast`, `dt-fastest`, `mt-normal`, `mt-fast`, `mt-fastest`

## Library usage
```rust
use ggwave_rs::{default_parameters, GgWave, ProtocolId};

let params = default_parameters();
let ggwave = GgWave::new(params)?;
let waveform = ggwave.encode(
    b"ping",
    ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_FAST,
    25,
)?;

if let Some(payload) = ggwave.decode(&waveform)? {
    println!("decoded: {}", String::from_utf8_lossy(&payload));
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Updating the submodule
```sh
git submodule update --remote --merge
```

## License
This crate is MIT licensed. Upstream ggwave is also MIT licensed.

