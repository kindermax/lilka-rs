use core::cell::RefCell;

use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};
use esp_hal::rtc_cntl::Rtc;
use jiff::Timestamp;

static CLOCK: Mutex<CriticalSectionRawMutex, RefCell<Option<Rtc<'static>>>> =
    Mutex::new(RefCell::new(None));

pub struct ClockService;

impl ClockService {
    pub fn init(rtc: Rtc<'static>) {
        CLOCK.lock(|inner| inner.borrow_mut().replace(rtc));
    }

    pub fn get_current_time() -> Timestamp {
        CLOCK.lock(|inner| {
            let rtc = inner.borrow();
            let rtc = rtc.as_ref().expect("ClockService not initialized");
            Timestamp::from_microsecond(rtc.current_time_us() as i64).unwrap()
        })
    }

    pub fn set_current_time(timestamp_us: u64) {
        CLOCK.lock(|inner| {
            let mut rtc = inner.borrow_mut();
            let rtc = rtc.as_mut().expect("ClockService not initialized");
            rtc.set_current_time_us(timestamp_us);
        });
    }
}
