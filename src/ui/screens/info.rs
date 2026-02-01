use crate::display::LilkaDisplay;
use crate::state::ButtonEvent;
use crate::ui::widgets::Header;
use crate::ui::{Screen, Transition, UIState};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{
    mono_font::iso_8859_10::FONT_10X20, mono_font::MonoTextStyle, pixelcolor::Rgb565, prelude::*,
    text::Text,
};
use embedded_layout::prelude::*;

pub struct InfoScreen {
    header: Header,
    display_bounds: Rectangle,
}

impl InfoScreen {
    pub fn new(display_bounds: Rectangle) -> Self {
        Self {
            header: Header::new(display_bounds),
            display_bounds,
        }
    }
}

impl Screen for InfoScreen {
    fn update(&mut self, event: ButtonEvent) -> Transition {
        match event {
            ButtonEvent::B => Transition::Pop,
            _ => Transition::Stay,
        }
    }

    fn draw(&mut self, display: &mut LilkaDisplay, state: &UIState) {
        display.clear(Rgb565::BLACK).unwrap();
        self.header.draw(display, state).unwrap();

        let content_area = Rectangle::new(
            Point::new(0, 30),
            Size::new(
                self.display_bounds.size.width,
                self.display_bounds.size.height - 30,
            ),
        );

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        Text::new("Info Screen", Point::zero(), text_style)
            .align_to(&content_area, horizontal::Center, vertical::Center)
            .draw(display)
            .unwrap();
    }
}

