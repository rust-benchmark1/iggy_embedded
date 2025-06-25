# iggy-embedded

Main goal of this project is to tinker with [Iggy](https://github.com/apache/iggy) and learn more about writing embedded software in Rust.

Project offers simple `no_std` TCP client allowing users to interact with Iggy TCP server.

Currently, the only supported board is ESP32. Repository was generated using [esp-generate](https://github.com/esp-rs/esp-generate).

## Setup

Make sure you have the following installed:

-   Rust toolchain
-   Espressif toolchain - you can read more about it in [The Rust on ESP Book](https://docs.esp-rs.org/book/introduction.html)
    -   [cargo-espflash](https://github.com/esp-rs/espflash/tree/main/cargo-espflash)
    -   [espflash](https://github.com/esp-rs/espflash/tree/main/espflash)
    -   [espup](https://github.com/esp-rs/espup)
-   Docker

## Usage

Before compiling the project, make sure you start the Iggy server in Docker:

```bash
docker compose up
```

Copy `cfg.toml.example` to `cfg.toml` and adjust the configuration to your needs. Make sure you set the correct:

-   WiFi credentials
-   Iggy TCP Server address
-   Iggy user credentials

To compile the project, run:

```bash
cargo run --release
```

It will automatically flash the binary to the board, so make sure you have it connected to your computer. Wait for the flashing process to finish and you should see the output in the terminal.
