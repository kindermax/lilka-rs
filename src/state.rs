pub const BUTTON_CHANNEL_SIZE: usize = 10;

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