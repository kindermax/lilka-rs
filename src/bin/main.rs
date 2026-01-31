#![no_std]
#![no_main]

use alloc::boxed::Box;
use alloc::vec::Vec;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use esp_backtrace as _;
use esp_hal::gpio::Input;
use log::info;
use embedded_graphics::prelude::Dimensions;

use lilka_rs::board::Board;
use lilka_rs::state::ButtonEvent;
use lilka_rs::state::BUTTON_CHANNEL_SIZE;
use lilka_rs::display::LilkaDisplay;
use lilka_rs::ui::{Screen, Transition};
use lilka_rs::ui::screens::MenuScreen;

extern crate alloc;

// Create a channel for button events
static BUTTON_CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, BUTTON_CHANNEL_SIZE> = Channel::new();

// up, down, left, right, a, b, c, d
const BUTTON_COUNT: usize = 8;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    // Initialize Hardware via the Board abstraction
    let board = Board::init();
    info!("Hardware initialized!");

    // Spawn Input Handlers
    let sender = BUTTON_CHANNEL.sender();
    spawner.spawn(button_handler(board.up, ButtonEvent::Up, sender)).unwrap();
    spawner.spawn(button_handler(board.down, ButtonEvent::Down, sender)).unwrap();
    spawner.spawn(button_handler(board.left, ButtonEvent::Left, sender)).unwrap();
    spawner.spawn(button_handler(board.right, ButtonEvent::Right, sender)).unwrap();
    spawner.spawn(button_handler(board.a, ButtonEvent::A, sender)).unwrap();
    spawner.spawn(button_handler(board.b, ButtonEvent::B, sender)).unwrap();

    // Spawn UI System
    spawner.spawn(ui_task(board.display, BUTTON_CHANNEL.receiver())).unwrap();

    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}

#[embassy_executor::task(pool_size = BUTTON_COUNT)]
async fn button_handler(
    mut button: Input<'static>,
    event: ButtonEvent,
    sender: Sender<'static, CriticalSectionRawMutex, ButtonEvent, BUTTON_CHANNEL_SIZE>
) {
    loop {
        button.wait_for_falling_edge().await;
        if button.is_low() {
            sender.send(event).await;
        }
        Timer::after(Duration::from_millis(50)).await;
        button.wait_for_rising_edge().await;
    }
}

#[embassy_executor::task]
async fn ui_task(
    mut display: LilkaDisplay,
    receiver: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, BUTTON_CHANNEL_SIZE>,
) {
    let mut stack: Vec<Box<dyn Screen>> = Vec::new();
    stack.push(Box::new(MenuScreen::new(display.bounding_box())));

    if let Some(screen) = stack.last_mut() {
        screen.draw(&mut display);
    }

    loop {
        let event = receiver.receive().await;
        info!("event: {:?}", event);

        let transition = if let Some(screen) = stack.last_mut() {
            screen.update(event)
        } else {
            Transition::Stay
        };

        match transition {
            Transition::Push(new_screen) => stack.push(new_screen),
            Transition::Pop => { stack.pop(); },
            Transition::Replace(new_screen) => {
                stack.pop();
                stack.push(new_screen);
            }
            Transition::Stay => {}
        }

        if let Some(screen) = stack.last_mut() {
            screen.draw(&mut display);
        } else {
            stack.push(Box::new(MenuScreen::new(display.bounding_box())));
            stack.last_mut().unwrap().draw(&mut display);
        }
    }
}
