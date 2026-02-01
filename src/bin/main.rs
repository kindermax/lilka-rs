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
use log::info;

use lilka_rs::board::Board;
use lilka_rs::display::LilkaDisplay;
use lilka_rs::input::{get_events, ButtonSet, InputPins};
use lilka_rs::state::ButtonEvent;
use lilka_rs::state::BUTTON_CHANNEL_SIZE;
use lilka_rs::ui::screens::MenuScreen;
use lilka_rs::ui::{Clock, Screen, Transition, UIState};

extern crate alloc;

// Create a channel for button events
static BUTTON_CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, BUTTON_CHANNEL_SIZE> =
    Channel::new();

#[esp_hal_embassy::main]
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

    let mut clock_service = ClockService::new(board.rtc);

    // Spawn Single Input System
    spawner
        .spawn(input_task(pins, BUTTON_CHANNEL.sender()))
        .unwrap();

    // Spawn UI System
    spawner
        .spawn(ui_task(
            board.display,
            clock_service,
            BUTTON_CHANNEL.receiver(),
        ))
        .unwrap();

    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}

#[embassy_executor::task]
async fn input_task(
    pins: InputPins,
    sender: Sender<'static, CriticalSectionRawMutex, ButtonEvent, BUTTON_CHANNEL_SIZE>,
) {
    let mut last_state = ButtonSet(0);

    loop {
        let current_state = pins.read_all();

        if current_state != last_state {
            // Convert bitmask changes into UI events
            for event in get_events(last_state, current_state) {
                sender.send(event).await;
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
    mut clock_service: ClockService,
    receiver: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, BUTTON_CHANNEL_SIZE>,
) {
    let mut stack: Vec<Box<dyn Screen>> = Vec::new();
    stack.push(Box::new(MenuScreen::new(display.bounding_box())));

    let mut state = UIState::default();

    if let Some(screen) = stack.last_mut() {
        screen.draw(&mut display, &state);
    }

    loop {
        let event = receiver.receive().await;
        info!("event: {:?}", event);
        state.clock.timestamp = clock_service.get_current_time();

        let transition = if let Some(screen) = stack.last_mut() {
            screen.update(event)
        } else {
            Transition::Stay
        };

        match transition {
            Transition::Push(new_screen) => stack.push(new_screen),
            Transition::Pop => {
                stack.pop();
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
