pub mod screens;
pub mod widgets;

use alloc::boxed::Box;
use crate::display::LilkaDisplay;
use crate::state::ButtonEvent;

/// Transitions tell the navigator what to do after a screen update.
pub enum Transition {
    /// Stay on the current screen.
    Stay,
    /// Push a new screen onto the stack.
    Push(Box<dyn Screen>),
    /// Pop the current screen and return to the previous one.
    Pop,
    /// Replace the current screen with a new one.
    Replace(Box<dyn Screen>),
}

/// The core trait for all UI screens.
pub trait Screen {
    /// Handle input and return a transition.
    fn update(&mut self, event: ButtonEvent) -> Transition;
    
    /// Draw the screen content.
    fn draw(&mut self, display: &mut LilkaDisplay);
}
