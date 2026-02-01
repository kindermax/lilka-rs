use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::Display;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use esp_hal::Blocking;

pub type LilkaDisplay = Display<
    SpiInterface<
        'static,
        SpiDevice<'static, NoopRawMutex, Spi<'static, Blocking>, Output<'static>>,
        // ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, Delay>,
        Output<'static>,
    >,
    ST7789,
    Output<'static>,
    // NoResetPin,
>;
