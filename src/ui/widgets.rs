use core::fmt::Write;
use embedded_graphics::mono_font::iso_8859_10::FONT_10X20;
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, RgbColor, Size},
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::Text,
    Drawable,
};
use embedded_layout::{
    align::{horizontal, vertical, Align},
    View,
};
use jiff::tz::TimeZone;

use crate::format;
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
        battery.draw(display)?;

        Ok(())
    }

    pub fn draw_clock<D>(&self, display: &mut D, state: &UIState) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let time = state.clock.timestamp.to_zoned(TimeZone::UTC);

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
