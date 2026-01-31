use alloc::boxed::Box;
use embedded_graphics::primitives::Rectangle;
use crate::display::LilkaDisplay;
use crate::state::ButtonEvent;
use crate::ui::{Screen, Transition};
use crate::ui::widgets::Header;
use crate::ui::screens::{InfoScreen, WifiScreen};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
};
use embedded_layout::prelude::*;
use embedded_menu::{
    interaction::{programmed::Programmed, Interaction, Navigation},
    items::MenuItem,
    theme::Theme,
    Menu, MenuStyle,
};
use embedded_graphics::mono_font::iso_8859_10::FONT_10X20;

#[derive(Copy, Clone)]
pub struct MenuColor {
    pub main_color: Rgb565,
    pub selected_text_color: Rgb565,
}

impl Theme for MenuColor {
    type Color = Rgb565;
    fn text_color(&self) -> Self::Color { self.main_color }
    fn selected_text_color(&self) -> Self::Color { self.selected_text_color }
    fn selection_color(&self) -> Self::Color { self.main_color }
}

pub type MainMenuType = Menu<
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

pub struct MenuScreen {
    menu: MainMenuType,
    header: Header,
    selected_idx: usize,
    display_bounds: Rectangle,
}

impl MenuScreen {
    pub fn new(display_bounds: Rectangle) -> Self {
        let style = MenuStyle::new(MenuColor {
            main_color: Rgb565::new(51, 255, 153),
            selected_text_color: Rgb565::RED,
        })
        .with_font(&FONT_10X20)
        .with_title_font(&FONT_10X20);

        let menu = Menu::with_style("", style)
            .add_menu_items([
                MenuItem::new("Info", false),
                MenuItem::new("Network", false),
            ])
            .build();

        Self {
            menu,
            header: Header::new(display_bounds),
            selected_idx: 0,
            display_bounds,
        }
    }
}

impl Screen for MenuScreen {
    fn update(&mut self, event: ButtonEvent) -> Transition {
        match event {
            ButtonEvent::Up => {
                self.menu.interact(Interaction::Navigation(Navigation::Previous));
                self.selected_idx = self.selected_idx.saturating_sub(1);
                Transition::Stay
            }
            ButtonEvent::Down => {
                self.menu.interact(Interaction::Navigation(Navigation::Next));
                self.selected_idx = (self.selected_idx + 1) % 2;
                Transition::Stay
            }
            ButtonEvent::Right | ButtonEvent::A => {
                match self.selected_idx {
                    0 => Transition::Push(Box::new(InfoScreen::new(self.display_bounds))),
                    1 => Transition::Push(Box::new(WifiScreen::new(self.display_bounds))),
                    _ => Transition::Stay,
                }
            }
            _ => Transition::Stay,
        }
    }

    fn draw(&mut self, display: &mut LilkaDisplay) {
        display.clear(Rgb565::BLACK).unwrap();
        self.header.draw(display).unwrap();

        // Define the "Slot" for the menu: everything below the 30px header
        let menu_area = Rectangle::new(
            Point::new(0, 30),
            Size::new(self.display_bounds.size.width, self.display_bounds.size.height - 30),
        );

        // Use the MenuDisplay wrapper from crate::menu to provide clipping and relative positioning
        let mut menu_display = crate::menu::MenuDisplay::new(display, menu_area);

        self.menu.update(&menu_display);
        self.menu.draw(&mut menu_display).unwrap();
    }
}