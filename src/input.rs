use crate::state::ButtonEvent;
use esp_hal::gpio::Input;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ButtonSet(pub u16);

impl ButtonSet {
    pub const UP: u16 = 1 << 0;
    pub const DOWN: u16 = 1 << 1;
    pub const LEFT: u16 = 1 << 2;
    pub const RIGHT: u16 = 1 << 3;
    pub const A: u16 = 1 << 4;
    pub const B: u16 = 1 << 5;
    pub const C: u16 = 1 << 6;
    pub const D: u16 = 1 << 7;

    pub fn is_pressed(&self, mask: u16) -> bool {
        (self.0 & mask) != 0
    }
}

pub struct InputPins {
    pub up: Input<'static>,
    pub down: Input<'static>,
    pub left: Input<'static>,
    pub right: Input<'static>,
    pub a: Input<'static>,
    pub b: Input<'static>,
    pub c: Input<'static>,
    pub d: Input<'static>,
}

impl InputPins {
    pub fn read_all(&self) -> ButtonSet {
        let mut bits = 0u16;
        if self.up.is_low() {
            bits |= ButtonSet::UP;
        }
        if self.down.is_low() {
            bits |= ButtonSet::DOWN;
        }
        if self.left.is_low() {
            bits |= ButtonSet::LEFT;
        }
        if self.right.is_low() {
            bits |= ButtonSet::RIGHT;
        }
        if self.a.is_low() {
            bits |= ButtonSet::A;
        }
        if self.b.is_low() {
            bits |= ButtonSet::B;
        }
        if self.c.is_low() {
            bits |= ButtonSet::C;
        }
        if self.d.is_low() {
            bits |= ButtonSet::D;
        }
        ButtonSet(bits)
    }
}

/// Helper to convert a bitmask change into discrete events for the UI.
/// This allows us to keep the existing UI logic while using the new scanner.
pub fn get_events(old: ButtonSet, new: ButtonSet) -> impl Iterator<Item = ButtonEvent> {
    let mut events = [None; 8];
    let mut i = 0;

    let changed = new.0 & !old.0; // Only buttons that JUST went low (pressed)

    if (changed & ButtonSet::UP) != 0 {
        events[i] = Some(ButtonEvent::Up);
        i += 1;
    }
    if (changed & ButtonSet::DOWN) != 0 {
        events[i] = Some(ButtonEvent::Down);
        i += 1;
    }
    if (changed & ButtonSet::LEFT) != 0 {
        events[i] = Some(ButtonEvent::Left);
        i += 1;
    }
    if (changed & ButtonSet::RIGHT) != 0 {
        events[i] = Some(ButtonEvent::Right);
        i += 1;
    }
    if (changed & ButtonSet::A) != 0 {
        events[i] = Some(ButtonEvent::A);
        i += 1;
    }
    if (changed & ButtonSet::B) != 0 {
        events[i] = Some(ButtonEvent::B);
        i += 1;
    }
    if (changed & ButtonSet::C) != 0 {
        events[i] = Some(ButtonEvent::C);
        i += 1;
    }
    if (changed & ButtonSet::D) != 0 {
        events[i] = Some(ButtonEvent::D);
        i += 1;
    }

    events.into_iter().flatten()
}
