use embassy_time::{Duration, Timer};
use esp_hal::{
    ledc::{
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
        Ledc, LowSpeed,
    },
    peripherals::GPIO11,
    time::Rate,
};

use crate::music;

pub struct Buzzer {
    output_pin: GPIO11<'static>,
}

impl Buzzer {
    pub fn new(pin: GPIO11<'static>) -> Self {
        Buzzer { output_pin: pin }
    }

    pub async fn play(&mut self, freq: f64, duration: u64, ledc: &mut Ledc<'_>) {
        let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
        lstimer0
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty8Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: Rate::from_hz(freq as u32),
            })
            .unwrap();

        let mut channel = ledc.channel(channel::Number::Channel0, self.output_pin.reborrow());
        channel
            .configure(channel::config::Config {
                timer: &lstimer0,
                duty_pct: 50,
                drive_mode: esp_hal::gpio::DriveMode::PushPull, // pin_config: channel::config::PinConfig::PushPull,
            })
            .unwrap();

        Timer::after(Duration::from_millis(duration)).await;
        channel.set_duty(0).unwrap();
    }

    // TODO: mutex ?
    pub async fn play_song<'s>(&mut self, song: &music::Song<'s>, ledc: &mut Ledc<'_>) {
        for (note, duration_type) in song.melody {
            let note_duration = song.calc_note_duration(*duration_type) as u64;
            let pause_duration = note_duration / 10; // 10% of note_duration
            if *note == music::notes::REST {
                Timer::after(Duration::from_millis(note_duration)).await;
                continue;
            }
            self.play(*note, note_duration - pause_duration, ledc).await;
            Timer::after(Duration::from_millis(pause_duration)).await;
        }
    }
}
