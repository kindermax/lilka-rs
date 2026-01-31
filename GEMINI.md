# lilka-rs Project Context

## Project Overview

`lilka-rs` is a Rust-based firmware/kernel for the "Lilka" embedded device, powered by the ESP32-S3 microcontroller. It utilizes the Embassy async ecosystem for efficient, non-blocking task management.

**Key Technologies:**
*   **Language:** Rust (no_std)
*   **Microcontroller:** ESP32-S3 (`xtensa-esp32s3-none-elf`)
*   **Framework:** Embassy (`embassy-executor`, `embassy-time`, `esp-hal-embassy`)
*   **HAL:** `esp-hal` (Hardware Abstraction Layer)
*   **Graphics:** `embedded-graphics`, `mipidsi` (Display driver for ST7789), `embedded-menu`
*   **Build System:** Cargo with `espflash` as the runner.

## Architecture

*   **Entry Point:** `src/bin/main.rs`
    *   Initializes the ESP32-S3 peripherals (Clocks, GPIO, SPI, Timer).
    *   Sets up the Display (ST7789 via SPI).
    *   Initializes Buttons (GPIO inputs with interrupts/async wait).
    *   Spawns async tasks using the Embassy executor.
*   **Tasks:**
    *   `button_handler`: Monitors button presses (debouncing included) and sends events via a channel.
    *   `ui_task`: Receives button events and manages the UI state (Menu, Info, Wifi screens) and rendering.
*   **Modules (`src/`):**
    *   `lib.rs`: Module exports.
    *   `display.rs`: Display configuration/abstraction.
    *   `menu.rs`: Menu structure and rendering logic.
    *   `state.rs`: State management definitions (e.g., `ButtonEvent`).
    *   `buzzer.rs`: Buzzer control (likely for audio feedback/music).
    *   `music/`: Music playback logic.

## Building and Running

The project requires the ESP Rust toolchain.

**Prerequisites:**
*   `espup` (to install the toolchain)
*   `espflash` (for flashing via USB)
*   `probe-rs` (optional, for flashing/debugging via debug probe)

**Commands:**

*   **Build:**
    ```bash
    cargo build
    ```

*   **Flash & Monitor (USB):**
    This project is configured to use `espflash` as the runner.
    ```bash
    cargo run
    ```
    *   To specify a port: `cargo run -- --port /dev/ttyACM0` (adjust port as needed).

*   **Flash (via probe-rs):**
    ```bash
    cargo flash --chip esp32s3
    ```

## Development Conventions

*   **Async/Await:** The project relies heavily on Rust's `async`/`await` syntax via Embassy. All hardware interaction that might block should be async.
*   **No Standard Library:** This is a `#![no_std]` environment. Standard library features are not available; use `core` and `alloc` (if heap is initialized).
*   **Logging:** `esp-println` and `log` crate are used. Use `info!`, `warn!`, `error!` macros.
*   **Memory:** Global static resources are often managed using `StaticCell` and passed to tasks or drivers.
*   **Configuration:** `Cargo.toml` manages dependencies and feature flags for the specific hardware (ESP32-S3). `.cargo/config.toml` handles target-specific build flags.
