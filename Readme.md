# Toy lilka kernel written in Rust

## Development (only for myself)

Install toolchain

See: https://rust.lilka.dev/creating_rom.html

```
# outside of this project run:
cargo install espup --locked
espup install
cargo install espflash@3.3.0 --locked (use 4.x when migrate to esp-hal 1.0.0-rc.0)
```

Run IDE
```
cd ~/code/projects/lilka-rs
esp-rs-setup
code .
```

## Wi-Fi credentials

Wi-Fi credentials are supplied at compile time and must not be committed:

```sh
cp .env.example .env
# Edit .env, then load it into the current shell.
source .env
cargo build
```

The generated firmware contains these values, so do not distribute firmware
images built with real credentials.


## Flush firmware

1. Turn off the board (switch or disconnect usb)
2. Press and hold Select button 
3. Turn on the board (switch or connect usb)
4. Flush using probe-rs `cargo build && cargo flash --chip esp32s3` command 
    or espflush `cargo run` or `cargo run -- --port /dev/cu.usbmodem112201` if want to specify port
