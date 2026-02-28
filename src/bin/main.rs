#![no_std]
#![no_main]

use alloc::boxed::Box;
use alloc::vec::Vec;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};

use embedded_graphics::prelude::Dimensions;
use esp_backtrace as _;
use esp_println::println;
use log::info;

use lilka_rs::board::Board;
use lilka_rs::display::LilkaDisplay;
use lilka_rs::input::{get_events, ButtonSet, InputPins};
use lilka_rs::services::ntp_task;
use lilka_rs::services::{network_task, ClockService, NetworkService};
use lilka_rs::state::{UIEvent, UI_CHANNEL_SIZE};
use lilka_rs::ui::screens::MenuScreen;
use lilka_rs::ui::{Screen, Transition, UIState};

extern crate alloc;

// Create a channel for UI events (buttons + ticks)
static UI_CHANNEL: Channel<CriticalSectionRawMutex, UIEvent, UI_CHANNEL_SIZE> = Channel::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    // Initialize Hardware
    let board = Board::init();
    info!("Hardware initialized!");

    // Group pins for the single input scanner
    let pins = InputPins {
        up: board.up,
        down: board.down,
        left: board.left,
        right: board.right,
        a: board.a,
        b: board.b,
        c: board.c,
        d: board.d,
    };

    ClockService::init(board.rtc);
    spawner.spawn(network_task(board.wifi)).unwrap();
    spawner.spawn(ntp_task("pool.ntp.org")).unwrap();

    // Spawn tick task for 1-second UI updates
    spawner.spawn(tick_task(UI_CHANNEL.sender())).unwrap();

    // Spawn Single Input System
    spawner
        .spawn(input_task(pins, UI_CHANNEL.sender()))
        .unwrap();

    // Spawn UI System
    spawner
        .spawn(ui_task(board.display, UI_CHANNEL.receiver()))
        .unwrap();

    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}

#[embassy_executor::task]
async fn tick_task(sender: Sender<'static, CriticalSectionRawMutex, UIEvent, UI_CHANNEL_SIZE>) {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        sender.send(UIEvent::Tick).await;
    }
}

#[embassy_executor::task]
async fn input_task(
    pins: InputPins,
    sender: Sender<'static, CriticalSectionRawMutex, UIEvent, UI_CHANNEL_SIZE>,
) {
    let mut last_state = ButtonSet(0);

    loop {
        let current_state = pins.read_all();

        if current_state != last_state {
            // Convert bitmask changes into UI events
            for event in get_events(last_state, current_state) {
                sender.send(UIEvent::Button(event)).await;
            }
            last_state = current_state;
        }

        // 20ms poll rate (50Hz) is plenty for UI and provides natural debouncing
        Timer::after(Duration::from_millis(20)).await;
    }
}

#[embassy_executor::task]
async fn ui_task(
    mut display: LilkaDisplay,
    receiver: Receiver<'static, CriticalSectionRawMutex, UIEvent, UI_CHANNEL_SIZE>,
) {
    let mut stack: Vec<Box<dyn Screen>> = Vec::new();
    stack.push(Box::new(MenuScreen::new(display.bounding_box())));

    let mut state = UIState::default();

    if let Some(screen) = stack.last_mut() {
        screen.draw(&mut display, &state);
    }

    loop {
        let event = receiver.receive().await;

        // Update state
        state.wifi_connected = NetworkService::stack()
            .map(|s| s.is_link_up() && s.is_config_up())
            .unwrap_or(false);

        // Only process screen transitions on button events
        let transition = match event {
            UIEvent::Button(button_event) => {
                info!("button: {:?}", button_event);
                if let Some(screen) = stack.last_mut() {
                    screen.update(button_event)
                } else {
                    Transition::Stay
                }
            }
            UIEvent::Tick => Transition::Stay,
        };

        match transition {
            Transition::Push(new_screen) => stack.push(new_screen),
            Transition::Pop => {
                stack.pop();
                if let Some(screen) = stack.last_mut() {
                    screen.ensure_redraw();
                }
            }
            Transition::Replace(new_screen) => {
                stack.pop();
                stack.push(new_screen);
            }
            Transition::Stay => {}
        }

        if let Some(screen) = stack.last_mut() {
            screen.draw(&mut display, &state);
        } else {
            stack.push(Box::new(MenuScreen::new(display.bounding_box())));
            stack.last_mut().unwrap().draw(&mut display, &state);
        }
    }
}
