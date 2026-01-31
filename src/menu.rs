use core::convert::Infallible;
use core::ops::{Deref, DerefMut};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget},
    primitives::{Rectangle},
};
use embedded_graphics::geometry::Dimensions;
use embassy_embedded_hal::shared_bus::SpiDeviceError;
use mipidsi::interface::SpiError;

use crate::{display::LilkaDisplay};

/// MenuDisplay is a wrapper around the display that allows us to draw the menu on it
/// and also allows to specify the bounds of the menu
pub struct MenuDisplay<'a> {
    display: &'a mut LilkaDisplay,
    bounds: Rectangle,
}

impl<'a> MenuDisplay<'a> {
    pub fn new(display: &'a mut LilkaDisplay, bounds: Rectangle) -> Self {
        Self { display, bounds }
    }
}

// Implement Deref to delegate DrawTarget trait calls to the underlying display
impl<'a> Deref for MenuDisplay<'a> {
    type Target = LilkaDisplay;

    fn deref(&self) -> &Self::Target {
        self.display
    }
}

// Implement DerefMut to delegate mutable DrawTarget trait calls to the underlying display
impl<'a> DerefMut for MenuDisplay<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.display
    }
}

// Only implement Dimensions since DrawTarget is now handled by Deref/DerefMut
impl<'a> Dimensions for MenuDisplay<'a> {
    fn bounding_box(&self) -> Rectangle {
        self.bounds
    }
}

// Implement DrawTarget by delegating to the underlying display
impl<'a> DrawTarget for MenuDisplay<'a> {
    type Color = Rgb565;
    type Error = SpiError<SpiDeviceError<esp_hal::spi::Error, Infallible>, Infallible>;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>> {
        // Delegate to the underlying display using DerefMut
        self.deref_mut().draw_iter(pixels)
    }
}
