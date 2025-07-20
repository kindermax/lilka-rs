#![no_std]
#![no_main]

use core::cell::RefCell;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::iso_8859_10::FONT_10X20;
// use embedded_graphics::mono_font::iso_8859_5::FONT_9X18_BOLD;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::prelude::*;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::primitives::Line;

use embedded_text::style::TextBoxStyleBuilder;
use embedded_text::TextBox;

// use esp_hal_embassy::init;
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
// TODO: why blocking ?
use embassy_embedded_hal::shared_bus::blocking::spi::{SpiDevice, SpiDeviceWithConfig};
// use embedded_hal_bus::spi::ExclusiveDevice;
use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};

use esp_backtrace as _;

use esp_hal::time::Rate;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, NoPin, Output, OutputConfig};
use esp_hal::ledc::{self, Ledc};
// use esp_hal::peripherals::SPI2;
use esp_hal::spi::master::{Spi, Config};
use esp_hal::{spi, Blocking};
use esp_hal::timer::systimer::SystemTimer;

use mipidsi::{Display, NoResetPin};
use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, Orientation, RefreshOrder, Rotation};
use mipidsi::Builder;
use mipidsi::models::ST7789;

use log::info;

// TODO: replace with crate::
use lilka_rs::buzzer::Buzzer;
use lilka_rs::music::{Song, songs::startup};
use static_cell::StaticCell;

extern crate alloc;

static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, Blocking>>>> = StaticCell::new();
static DISPLAY_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    // let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_160MHz);
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);
    //fff
    use esp_hal::timer::timg::TimerGroup;

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    ///fff

    // let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    // esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    // let mut ledc = Ledc::new(peripherals.LEDC);
    // ledc.set_global_slow_clock(ledc::LSGlobalClkSource::APBClk);

    // let mut buzzer = Buzzer::new(peripherals.GPIO11);
    // let song = Song::new(startup::TEMPO, &startup::MELODY);
    // buzzer.play_song(&song, &mut ledc).await;

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

    // render_splash(display).await;
    render_splash_v2(display).await;

    // loop {
    //     Timer::after_millis(10u64).await;
    //     text_box.draw(&mut display).unwrap();
    //     info!("Text drawn");
    // }
    // executor.run(|spawn| {
    // spawner.spawn(run(display)).unwrap();
    // });

    loop {}
}

type LilkaDisplay = Display<
    SpiInterface<
        'static,
        SpiDevice<'static, NoopRawMutex, Spi<'static, Blocking>, Output<'static>>,
        // ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, Delay>,
        Output<'static>
    >,
    ST7789,
    Output<'static>,
    // NoResetPin,
>;

#[embassy_executor::task]
async fn run(mut display: LilkaDisplay) {

    render_splash(display).await;
}

async fn render_splash(mut display: LilkaDisplay) {
    let char_color = Rgb565::new(51, 255, 153);
    let char_style = MonoTextStyle::new(&FONT_10X20, char_color);
    let textbox_style = TextBoxStyleBuilder::new()
    .height_mode(embedded_text::style::HeightMode::FitToText)
    .alignment(embedded_text::alignment::HorizontalAlignment::Center)
    .vertical_alignment(embedded_text::alignment::VerticalAlignment::Middle)
    .build();

    let bounds = Rectangle::new(
        Point::new(0, 130), Size::new(280, 0)
    );

    let start_text = "Lilka-rs XXX YYY";

    let text_box = TextBox::with_textbox_style(start_text, bounds, char_style, textbox_style);

    loop {
        Timer::after_millis(10u64).await;
        text_box.draw(&mut display).unwrap();
    }
}

use embedded_graphics::{
    // pixelcolor::PixelColor,
    // pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle},
    text::Text,
};
use embedded_layout::{layout::linear::LinearLayout, prelude::*};

pub struct Header {
    bounds: Rectangle,
}
impl Header {
    /// The header has a configurable position and size
    fn new(position: Point, size: Size) -> Self {
        Self {
            bounds: Rectangle::new(position, size),
        }
    }
}

impl View for Header {
    #[inline]
    fn translate_impl(&mut self, by: Point) {
        // make sure you don't accidentally call `translate`!
        <Rectangle as embedded_graphics::prelude::Transform>::translate_mut(&mut self.bounds, by);
    }

    #[inline]
    fn bounds(&self) -> Rectangle {
        self.bounds
    }
}

impl Drawable for Header {
    type Color = Rgb565;
    type Output = ();

    fn draw<D: DrawTarget<Color = Rgb565>>(&self, display: &mut D) -> Result<(), D::Error> {
        // Create styles
        let color = Rgb565::new(51, 255, 153);
        let line_style = PrimitiveStyle::with_stroke(color, 1);

        // Create only a bottom line for the header
        let bottom_left = Point::new(self.bounds.top_left.x, self.bounds.top_left.y + self.bounds.size.height as i32 - 1);
        let bottom_right = Point::new(self.bounds.top_left.x + self.bounds.size.width as i32, self.bounds.top_left.y + self.bounds.size.height as i32 - 1);
        let bottom_line = Line::new(bottom_left, bottom_right).into_styled(line_style);

        let font = FONT_10X20;
        let char_color = Rgb565::new(51, 255, 153);
        let text_style = MonoTextStyle::new(&font, char_color);

        // Primitives to be displayed
        let time = Text::new("00:00", Point::zero(), text_style)
            .align_to(&self.bounds, horizontal::Left, vertical::Center)
            .translate(Point::new(20, 0));

        let battery = Text::new("100%", Point::zero(), text_style)
            .align_to(&self.bounds, horizontal::Right, vertical::Center)
            .translate(Point::new(-20, 0));

        let header_center = Text::new("Lilka", Point::zero(), text_style)
            .align_to(&self.bounds, horizontal::Center, vertical::Center);

        // Draw views - only the bottom line and text
        bottom_line.draw(display)?;
        time.draw(display)?;
        battery.draw(display)?;
        header_center.draw(display)?;

        Ok(())
    }
}


async fn render_splash_v2(mut display: LilkaDisplay) {
    // Create a Rectangle from the display's dimensions
    let display_area = display.bounding_box();

    let header = Header::new(Point::new(0, 0), Size::new(display_area.size().width, 30)); // Header with 30px height
    // let menu = Menu::new();

    // The layout
    // Header
    // line
    // menu view
    // menu items
    LinearLayout::vertical(
        Chain::new(header)
        // .append(menu)
    )
    .with_alignment(horizontal::Center)
    .arrange()
    .align_to(&display_area, horizontal::Center, vertical::Top)
    .draw(&mut display)
    .unwrap();
    // loop {
    //     Timer::after_millis(10u64).await;
    //     text_box.draw(&mut display).unwrap();
    // }
}