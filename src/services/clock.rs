use esp_hal::rtc_cntl::Rtc;
use jiff::Timestamp;

// TODO: ntp ?
pub struct ClockService {
    rtc: Rtc<'static>,
}

impl ClockService {
    pub fn new(rtc: Rtc<'static>) -> Self {
        ClockService { rtc }
    }
    /// Initialize the clock service.
    pub fn init() {
        // Implementation for initializing the clock service goes here.
        // This could involve setting up timers, synchronizing with an external time source, etc.
    }

    /// Get the current time as a formatted string.
    pub fn get_current_time(&self) -> Timestamp {
        Timestamp::from_microsecond(self.rtc.current_time_us() as i64).unwrap()
    }

    pub fn set_current_time(&mut self, timestamp: u64) {
        self.rtc.set_current_time_us(timestamp);
    }
}
