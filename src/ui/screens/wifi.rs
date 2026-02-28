use crate::display::LilkaDisplay;
use crate::state::ButtonEvent;
use crate::ui::{Screen, Transition, UIState};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{
    mono_font::iso_8859_10::FONT_10X20, mono_font::MonoTextStyle, pixelcolor::Rgb565, prelude::*,
    text::Text,
};
use embedded_layout::prelude::*;

pub struct WifiScreen {
    display_bounds: Rectangle,
    initial_draw: bool,
}

impl WifiScreen {
    pub fn new(display_bounds: Rectangle) -> Self {
        Self {
            display_bounds,
            initial_draw: true,
        }
    }
}

impl Screen for WifiScreen {
    fn update(&mut self, event: ButtonEvent) -> Transition {
        match event {
            ButtonEvent::B => Transition::Pop,
            _ => Transition::Stay,
        }
    }

    fn draw(&mut self, display: &mut LilkaDisplay, _state: &UIState) {
        if self.initial_draw {
            let content_area = Rectangle::new(
                Point::new(0, 30),
                Size::new(
                    self.display_bounds.size.width,
                    self.display_bounds.size.height - 30,
                ),
            );
            display.fill_solid(&content_area, Rgb565::BLACK).unwrap();

            let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
            Text::new("Wifi Config", Point::zero(), text_style)
                .align_to(&content_area, horizontal::Center, vertical::Center)
                .draw(display)
                .unwrap();

            self.initial_draw = false;
        }
    }

    fn ensure_redraw(&mut self) {
        self.initial_draw = true;
    }
}
