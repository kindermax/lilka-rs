use crate::music::notes::*;

// change this to make the song slower or faster
pub const TEMPO: u16 = 280;

// lilka startup melody
pub const MELODY: [(f64, i16); 6] = [
    (NOTE_C3, 8),
    (NOTE_C4, 8),
    (NOTE_C5, 8),
    (NOTE_C7, 4),
    (REST, 8),
    (NOTE_C6, 4),
];
