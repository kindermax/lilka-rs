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


## Flush firmware

1. Turn off the board (switch or disconnect usb)
2. Press and hold Select button 
3. Turn on the board (switch or connect usb)
4. Flush using probe-rs `cargo build && cargo flash --chip esp32s3` command 
    or espflush `cargo run`