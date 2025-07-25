use core::convert::Infallible;

use embassy_embedded_hal::shared_bus::SpiDeviceError;
use embedded_graphics::{geometry::AnchorPoint, mono_font::MonoTextStyle, pixelcolor::Rgb565, prelude::{DrawTarget, Point, Primitive, Size}, primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Styled}, text::Text, Drawable};
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::mono_font::iso_8859_10::FONT_10X20;
use embedded_layout::{align::{horizontal, vertical, Align}, View};
use embedded_menu::{interaction::{programmed::Programmed}, items::MenuItem, theme::Theme, Menu, MenuStyle, SelectValue};
use mipidsi::interface::SpiError;

use crate::{display::LilkaDisplay};


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

pub fn create_header(display_area: Rectangle) -> Header {
    // Header with 30px height
    let header = Header::new(Point::new(0, 0), Size::new(display_area.size().width, 30));
    header
}

#[derive(Copy, Clone, PartialEq, SelectValue)]
pub enum TestEnum {
    A,
    B,
    C,
}

#[derive(Copy, Clone)]
pub struct MenuColor {
    main_color: Rgb565,
    selected_text_color: Rgb565
}

impl Theme for MenuColor {
    type Color = Rgb565;

    fn text_color(&self) -> Self::Color {
        self.main_color
    }

    fn selected_text_color(&self) -> Self::Color {
        self.selected_text_color
    }

    fn selection_color(&self) -> Self::Color {
        self.main_color
    }
}

#[derive(Copy, Clone)]
pub enum Screen {
    MainMenu { idx: usize },
    Info,
    Wifi,
}

/// MenuDisplay is a wrapper around the display that allows us to draw the menu on it
/// and also allws to specify the bounds of the menu
struct MenuDisplay<'a> {
    display: &'a mut LilkaDisplay,
    bounds: Rectangle,
}

impl<'a> MenuDisplay<'a> {
    fn new(display: &'a mut LilkaDisplay, bounds: Rectangle) -> Self {
        Self { display, bounds }
    }
}

impl<'a> DrawTarget for MenuDisplay<'a> {
    type Color = Rgb565;
    type Error = SpiError<SpiDeviceError<esp_hal::spi::Error, Infallible>, Infallible>;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>> {
        self.display.draw_iter(pixels)
    }
}

impl<'a> Dimensions for MenuDisplay<'a> {
    fn bounding_box(&self) -> Rectangle {
        self.bounds
    }
}

type MainMenu = Menu<
    &'static str,
    Programmed,
    embedded_layout::prelude::Chain<
        embedded_menu::collection::MenuItems<
            [MenuItem<&'static str, (), bool, true>; 2],
            MenuItem<&'static str, (), bool, true>, ()
        >
    >,
    (),
    embedded_menu::selection_indicator::StaticPosition,
    embedded_menu::selection_indicator::style::Line,
    MenuColor
>;

pub fn create_menu() -> MainMenu {
    let style = MenuStyle::new(MenuColor {
        main_color: Rgb565::new(51, 255, 153),
        selected_text_color: Rgb565::new(255, 0, 0)
    })
    // .with_animated_selection_indicator(5) // Reduced animation duration from 10 to 5
    .with_font(&FONT_10X20)
    .with_title_font(&FONT_10X20);

    let menu_items = [
        MenuItem::new("Info", false),
        MenuItem::new("Network", false),
    ];

    // TOOD: no way I'l use this fake header, we need a real one
    let mut menu = Menu::with_style("", style)
        // .add_item("Foo", ">", |_| ())
        .add_menu_items(menu_items)
        // .add_item("Foo", "<-", |_| ())
        // .add_item("Check this", false, |_| ())
        // .add_item("Check this too", TestEnum::A, |_| ())
        .build();

    menu
}


pub async fn render_menu(
    display: &mut LilkaDisplay,
    menu: &mut MainMenu,
) {
    // TODO: this is not very performant creaging this structs  every time

    let border_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::new(255, 0, 0))
        .stroke_width(3)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();

    let menu_borders = display.bounding_box()
        .resized(Size::new(240, 200), AnchorPoint::BottomCenter)
        .into_styled(border_stroke);
    // TODO: fordebug
    menu_borders.draw(display).unwrap();

    let mut menu_display = MenuDisplay::new(display, menu_borders.primitive);

    menu.update(&menu_display);
    menu.draw(&mut menu_display).unwrap();
}