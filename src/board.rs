use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::NoopMutex;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, NoPin, Output, OutputConfig, Pull};
use esp_hal::spi::master::Spi;
use esp_hal::{spi, Blocking};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, RefreshOrder, Rotation};
use mipidsi::Builder;
use embassy_time::Delay;
use static_cell::StaticCell;

use crate::display::LilkaDisplay;

static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, Blocking>>>> = StaticCell::new();
static DISPLAY_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

pub struct Board {
    pub display: LilkaDisplay,
    pub up: Input<'static>,
    pub down: Input<'static>,
    pub left: Input<'static>,
    pub right: Input<'static>,
    pub a: Input<'static>,
    pub b: Input<'static>,
    pub c: Input<'static>,
    pub d: Input<'static>,
}

impl Board {
    pub fn init() -> Self {
        // 1. Initialize Peripherals
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);

        // 2. Initialize Heap (72 KB)
        esp_alloc::heap_allocator!(size: 72 * 1024);

        // 3. Initialize Embassy Timer
        let timg0 = TimerGroup::new(peripherals.TIMG0);
        esp_hal_embassy::init(timg0.timer0);

        // 4. Backlight
        Output::new(peripherals.GPIO46, Level::High, OutputConfig::default()).set_high();

        // 5. SPI & Display
        let spi_mosi = peripherals.GPIO17;
        let spi_clk = peripherals.GPIO18;
        let display_dc = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
        let display_cs = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());
        let display_rst = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

        let spi_config = spi::master::Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(spi::Mode::_0);

        let spi = Spi::new(peripherals.SPI2, spi_config).unwrap()
            .with_cs(NoPin)
            .with_mosi(spi_mosi)
            .with_miso(NoPin)
            .with_sck(spi_clk);

        let spi_bus = SPI_BUS.init(NoopMutex::new(RefCell::new(spi)));
        let spi_device = SpiDevice::new(spi_bus, display_cs);
        let display_buffer = DISPLAY_BUFFER.init([0_u8; 512]);
        let di = SpiInterface::new(spi_device, display_dc, display_buffer);

        let display = Builder::new(ST7789, di)
            .display_size(240, 280)
            .orientation(Orientation::new().rotate(Rotation::Deg270))
            .display_offset(0, 20)
            .refresh_order(RefreshOrder::default())
            .invert_colors(ColorInversion::Inverted)
            .reset_pin(display_rst)
            .init(&mut Delay)
            .unwrap();

        // 6. Buttons
        let controls_config = InputConfig::default().with_pull(Pull::Up);
        
        Self {
            display,
            up: Input::new(peripherals.GPIO38, controls_config),
            down: Input::new(peripherals.GPIO41, controls_config),
            left: Input::new(peripherals.GPIO39, controls_config),
            right: Input::new(peripherals.GPIO40, controls_config),
            a: Input::new(peripherals.GPIO5, controls_config),
            b: Input::new(peripherals.GPIO6, controls_config),
            c: Input::new(peripherals.GPIO10, controls_config),
            d: Input::new(peripherals.GPIO9, controls_config),
        }
    }
}
