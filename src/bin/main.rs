#![no_std]
#![no_main]

use core::cell::RefCell;

use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::prelude::*;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::primitives::PrimitiveStyleBuilder;

use embedded_graphics::primitives::StrokeAlignment;
use embedded_menu::interaction::Action;
use embedded_menu::interaction::Interaction;
use embedded_menu::interaction::Navigation;

use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use embassy_embedded_hal::shared_bus::blocking::spi::{SpiDevice};

use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::{NoopMutex};

use esp_backtrace as _;

use esp_hal::gpio::Pull;
use esp_hal::time::Rate;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, NoPin, Output, OutputConfig};
use esp_hal::spi::master::{Spi, Config};
use esp_hal::{spi, Blocking};

use lilka_rs::menu::create_header;
use lilka_rs::menu::create_menu;
use lilka_rs::menu::render_menu;
use lilka_rs::menu::Screen;
use lilka_rs::state::ButtonEvent;
use lilka_rs::state::BUTTON_CHANNEL_SIZE;
use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, Orientation, RefreshOrder, Rotation};
use mipidsi::Builder;
use mipidsi::models::ST7789;

use log::info;

use static_cell::StaticCell;

use lilka_rs::buzzer::Buzzer;
use lilka_rs::music::{Song, songs::startup};
use lilka_rs::display::LilkaDisplay;

extern crate alloc;

static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, Blocking>>>> = StaticCell::new();
static DISPLAY_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();


// Create a channel for button events
static BUTTON_CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, BUTTON_CHANNEL_SIZE> = Channel::new();

// up, down, left, right, a, b, c, d
const BUTTON_COUNT: usize = 8;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    // let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_160MHz);
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);
    use esp_hal::timer::timg::TimerGroup;

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    info!("Embassy initialized!");

    // Turn on screen backlight
    Output::new(peripherals.GPIO46, Level::High, OutputConfig::default()).set_high();
    let spi_mosi = peripherals.GPIO17;
    let spi_miso = peripherals.GPIO8;
    let spi_clk = peripherals.GPIO18;

    let display_dc = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let display_cs = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());
    let display_rst = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    let spi_config = spi::master::Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(spi::Mode::_0);

    let spi = Spi::new(
        peripherals.SPI2, spi_config
    ).unwrap()
        .with_cs(NoPin)
        .with_mosi(spi_mosi)
        // .with_miso(spi_miso)
        .with_miso(NoPin)
        .with_sck(spi_clk);

    let spi_bus = SPI_BUS.init(NoopMutex::new(RefCell::new(spi)));

    let spi_device = SpiDevice::new(spi_bus, display_cs);
    let display_buffer = DISPLAY_BUFFER.init([0_u8; 512]);
    let di = SpiInterface::new(spi_device, display_dc, display_buffer);
    let mut display = match Builder::new(ST7789, di)
        .display_size(240, 280)
        .orientation(Orientation::new().rotate(Rotation::Deg270))
        .display_offset(0, 20)
        .refresh_order(RefreshOrder::default())
        .invert_colors(ColorInversion::Inverted)  // TODO: why
        .reset_pin(display_rst)  // TODO: why
        .init(&mut Delay) {
            Ok(d) => d,
            Err(err) => {
                info!("Error while initializing display {:?}", err);
                panic!("Done");
            }
        };

    // display.set_orientation(Orientation::new().rotate(Rotation::Deg270));
    display.clear(Rgb565::BLACK).unwrap();

    info!("Display initialized");

    let controls_config = InputConfig::default().with_pull(Pull::Up);

    let up = Input::new(peripherals.GPIO38, controls_config);
    let down = Input::new(peripherals.GPIO41, controls_config);
    let left = Input::new(peripherals.GPIO39, controls_config);
    let right = Input::new(peripherals.GPIO40, controls_config);
    let a = Input::new(peripherals.GPIO5, controls_config);
    let b = Input::new(peripherals.GPIO6, controls_config);
    let c = Input::new(peripherals.GPIO10, controls_config);
    let d = Input::new(peripherals.GPIO9, controls_config);

    spawner.spawn(button_handler(up, ButtonEvent::Up, BUTTON_CHANNEL.sender())).unwrap();
    spawner.spawn(button_handler(down, ButtonEvent::Down, BUTTON_CHANNEL.sender())).unwrap();
    spawner.spawn(button_handler(left, ButtonEvent::Left, BUTTON_CHANNEL.sender())).unwrap();
    spawner.spawn(button_handler(right, ButtonEvent::Right, BUTTON_CHANNEL.sender())).unwrap();
    spawner.spawn(button_handler(a, ButtonEvent::A, BUTTON_CHANNEL.sender())).unwrap();
    spawner.spawn(ui_task(display, BUTTON_CHANNEL.receiver())).unwrap();
}

#[embassy_executor::task(pool_size = BUTTON_COUNT)]
async fn button_handler(
    mut button: Input<'static>,
    event: ButtonEvent,
    sender: Sender<'static, CriticalSectionRawMutex, ButtonEvent, BUTTON_CHANNEL_SIZE>
) {
    loop {
        // button.wait_for_any_edge().await;
        button.wait_for_falling_edge().await;
        if button.is_low() {
            info!("button pressed {:?}", event);
            // button pressed
            sender.send(event).await;
            info!("button sent");
        }
        // Debounce: ignore further edges for 50ms
        Timer::after(Duration::from_millis(50)).await;
        // Wait until release (rising edge) before next loop
        button.wait_for_rising_edge().await;
    }
}

#[embassy_executor::task]
async fn ui_task(
    mut display: LilkaDisplay,
    receiver: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, BUTTON_CHANNEL_SIZE>,
) {
    let border_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::new(255, 255, 255))
        .stroke_width(3)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();
    display
        .bounding_box()
        .resized(Size::new(240, 240), AnchorPoint::Center)
        .into_styled(border_stroke)
        .draw(&mut display).unwrap();

    let mut screen = Screen::MainMenu { idx: 0 };

    let mut header = create_header(display.bounding_box());
    header.draw(&mut display).unwrap();

    let mut menu = create_menu();
    render_menu(&mut display, &mut menu).await;

    // Handle button events
    loop {
        let event = receiver.receive().await;
        info!("event: {:?}", event);
        match event {
            ButtonEvent::Down => {
                menu.interact(Interaction::Navigation(Navigation::Next));
                render_menu(&mut display, &mut menu).await;
            }
            ButtonEvent::Up => {
                menu.interact(Interaction::Navigation(Navigation::Previous));
                render_menu(&mut display, &mut menu).await;
            }
            ButtonEvent::Left => {
                menu.interact(Interaction::Action(Action::Select));
                render_menu(&mut display, &mut menu).await;
            }
            ButtonEvent::Right => {
                menu.interact(Interaction::Action(Action::Select));
                render_menu(&mut display, &mut menu).await;
            }
            ButtonEvent::A => {
                display.clear(Rgb565::BLACK).unwrap();
            }
        }
    }
}
