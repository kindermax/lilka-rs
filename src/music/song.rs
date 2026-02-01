pub struct Song<'a> {
    whole_note: u32,
    pub melody: &'a [(f64, i16)],
}

impl<'a> Song<'a> {
    pub fn new(tempo: u16, melody: &'a [(f64, i16)]) -> Self {
        let whole_note = (60_000 * 4) / tempo as u32;
        Self { whole_note, melody }
    }

    pub fn calc_note_duration(&self, divider: i16) -> u32 {
        if divider > 0 {
            self.whole_note / divider as u32
        } else {
            let duration = self.whole_note / divider.unsigned_abs() as u32;
            (duration as f64 * 1.5) as u32
        }
    }
}
