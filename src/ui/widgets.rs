use crate::format;
use crate::ui::UIState;
use core::fmt::Write;
use embedded_graphics::mono_font::iso_8859_10::FONT_10X20;
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, Size},
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::Text,
    Drawable,
};
use embedded_layout::{
    align::{horizontal, vertical, Align},
    View,
};

pub struct Header {
    bounds: Rectangle,
}

impl Header {
    pub fn new(display_area: Rectangle) -> Self {
        Self {
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

// impl Drawable for Header {
//     type Color = Rgb565;
//     type Output = ();
//
//     fn draw<D: DrawTarget<Color = Rgb565>>(&self, display: &mut D) -> Result<(), D::Error> {
//         let color = Rgb565::new(51, 255, 153);
//         let line_style = PrimitiveStyle::with_stroke(color, 1);
//
//         let bottom_left = Point::new(
//             self.bounds.top_left.x,
//             self.bounds.top_left.y + self.bounds.size.height as i32 - 1,
//         );
//         let bottom_right = Point::new(
//             self.bounds.top_left.x + self.bounds.size.width as i32,
//             self.bounds.top_left.y + self.bounds.size.height as i32 - 1,
//         );
//         let bottom_line = Line::new(bottom_left, bottom_right).into_styled(line_style);
//
//         let text_style = MonoTextStyle::new(&FONT_10X20, color);
//
//         let time_text = format!("{:02}:{:02}", state.clock.hours, state.clock.minutes);
//         let time = Text::new("00:00", Point::zero(), text_style)
//             .align_to(&self.bounds, horizontal::Left, vertical::Center)
//             .translate(Point::new(20, 0));
//
//         let battery = Text::new("100%", Point::zero(), text_style)
//             .align_to(&self.bounds, horizontal::Right, vertical::Center)
//             .translate(Point::new(-20, 0));
//
//         let header_center = Text::new("Lilka X", Point::zero(), text_style).align_to(
//             &self.bounds,
//             horizontal::Center,
//             vertical::Center,
//         );
//
//         bottom_line.draw(display)?;
//         time.draw(display)?;
//         battery.draw(display)?;
//         header_center.draw(display)?;
//
//         Ok(())
//     }
// }

impl Header {
    // type Color = Rgb565;
    // type Output = ();

    pub fn draw<D>(&self, display: &mut D, state: &UIState) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let color = Rgb565::new(51, 255, 153);
        let line_style = PrimitiveStyle::with_stroke(color, 1);

        let bottom_left = Point::new(
            self.bounds.top_left.x,
            self.bounds.top_left.y + self.bounds.size.height as i32 - 1,
        );
        let bottom_right = Point::new(
            self.bounds.top_left.x + self.bounds.size.width as i32,
            self.bounds.top_left.y + self.bounds.size.height as i32 - 1,
        );
        let bottom_line = Line::new(bottom_left, bottom_right).into_styled(line_style);

        let text_style = MonoTextStyle::new(&FONT_10X20, color);

        let time_text = format!(5, "{:02}:{:02}", state.clock.hours, state.clock.minutes);
        let time = Text::new(&time_text, Point::zero(), text_style)
            .align_to(&self.bounds, horizontal::Left, vertical::Center)
            .translate(Point::new(20, 0));

        let battery = Text::new("100%", Point::zero(), text_style)
            .align_to(&self.bounds, horizontal::Right, vertical::Center)
            .translate(Point::new(-20, 0));

        let header_center = Text::new("Lilka X", Point::zero(), text_style).align_to(
            &self.bounds,
            horizontal::Center,
            vertical::Center,
        );

        bottom_line.draw(display)?;
        time.draw(display)?;
        battery.draw(display)?;
        header_center.draw(display)?;

        Ok(())
    }
}
