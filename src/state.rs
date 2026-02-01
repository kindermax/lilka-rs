pub const UI_CHANNEL_SIZE: usize = 10;

// Define button events
#[derive(Copy, Clone, Debug)]
pub enum ButtonEvent {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    C,
    D,
}

// UI events include button presses and periodic ticks
#[derive(Copy, Clone, Debug)]
pub enum UIEvent {
    Button(ButtonEvent),
    Tick,
}
