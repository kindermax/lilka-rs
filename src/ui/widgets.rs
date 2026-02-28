use core::fmt::Write;
use embedded_graphics::mono_font::iso_8859_10::FONT_10X20;
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::{Angle, DrawTarget, Point, Primitive, RgbColor, Size},
    primitives::{Arc, Line, PrimitiveStyle, Rectangle},
    text::Text,
    Drawable,
};
use embedded_layout::{
    align::{horizontal, vertical, Align},
    View,
};
use esp_println::println;
use jiff::tz::TimeZone;

use crate::format;
use crate::services::ClockService;
use crate::ui::UIState;

pub struct Header {
    color: Rgb565,
    text_style: MonoTextStyle<'static, Rgb565>,
    bounds: Rectangle,
}

impl Header {
    pub fn new(display_area: Rectangle) -> Self {
        let color = Rgb565::new(51, 255, 153);
        // Create a style with background color to ensure overwrites don't require clearing
        let text_style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(color)
            .background_color(Rgb565::BLACK)
            .build();

        Self {
            color,
            text_style,
            bounds: Rectangle::new(Point::new(0, 0), Size::new(display_area.size().width, 30)),
        }
    }
}

impl View for Header {
    #[inline]
    fn translate_impl(&mut self, by: Point) {
        <Rectangle as embedded_graphics::prelude::Transform>::translate_mut(&mut self.bounds, by);
    }

    #[inline]
    fn bounds(&self) -> Rectangle {
        self.bounds
    }
}

impl Header {
    pub fn draw<D>(&self, display: &mut D, state: &UIState) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let line_style = PrimitiveStyle::with_stroke(self.color, 1);

        let bottom_left = Point::new(
            self.bounds.top_left.x,
            self.bounds.top_left.y + self.bounds.size.height as i32 - 1,
        );
        let bottom_right = Point::new(
            self.bounds.top_left.x + self.bounds.size.width as i32,
            self.bounds.top_left.y + self.bounds.size.height as i32 - 1,
        );
        let bottom_line = Line::new(bottom_left, bottom_right).into_styled(line_style);

        let battery = Text::new("100%", Point::zero(), self.text_style)
            .align_to(&self.bounds, horizontal::Right, vertical::Center)
            .translate(Point::new(-20, 0));

        bottom_line.draw(display)?;
        self.draw_clock(display, state)?;
        self.draw_wifi(display, state)?;
        battery.draw(display)?;

        Ok(())
    }

    pub fn draw_wifi<D>(&self, display: &mut D, state: &UIState) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let icon_area = Rectangle::new(self.bounds.top_left + Point::new(10, 5), Size::new(20, 20));
        let center = icon_area.top_left + Point::new(10, 18);
        let style = PrimitiveStyle::with_stroke(self.color, 1);

        // Clear the icon area so stale pixels from previous frame are gone
        icon_area
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(display)?;

        // Draw dot at the bottom
        embedded_graphics::primitives::Circle::new(center - Point::new(1, 1), 3)
            .into_styled(PrimitiveStyle::with_fill(self.color))
            .draw(display)?;

        // Draw 3 curves
        for r in [6i32, 10i32, 14i32] {
            Arc::new(
                center - Point::new(r, r),
                r as u32 * 2,
                Angle::from_degrees(225.0),
                Angle::from_degrees(90.0),
            )
            .into_styled(style)
            .draw(display)?;
        }

        if !state.wifi_connected {
            Line::new(
                icon_area.top_left + Point::new(2, 2),
                icon_area.top_left + Point::new(18, 18),
            )
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 1))
            .draw(display)?;
        }

        Ok(())
    }

    pub fn draw_clock<D>(&self, display: &mut D, _state: &UIState) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let timestamp = ClockService::get_current_time();
        let time = timestamp.to_zoned(TimeZone::UTC);

        let time_text = format!(
            8,
            "{:02}:{:02}:{:02}",
            time.hour(),
            time.minute(),
            time.second()
        );
        let time_widget = Text::new(&time_text, Point::zero(), self.text_style).align_to(
            &self.bounds,
            horizontal::Center,
            vertical::Center,
        );

        time_widget.draw(display)?;
        Ok(())
    }
}
