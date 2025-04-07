use std::time::Duration;

const SECS_IN_DAY: u64 = 60 * 60 * 24;
const MICROS_IN_SEC: u64 = 1_000_000;
const MICROS_IN_DAY: u64 = SECS_IN_DAY * MICROS_IN_SEC;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    months: i32,
    days: i32,
    micros: i64,
}

impl Interval {
    pub const fn from_duration(duration: &Duration) -> Self {
        let micros = duration.as_micros();
        let days = (micros / MICROS_IN_DAY as u128) as i32;
        let micros = (micros % MICROS_IN_DAY as u128) as i64;
        Self {
            months: 0,
            days,
            micros,
        }
    }

    pub const fn from_micros(micros: u64) -> Self {
        Self {
            months: 0,
            days: (micros / MICROS_IN_DAY) as i32,
            micros: (micros % MICROS_IN_DAY) as i64,
        }
    }

    pub const fn from_millis(millis: u64) -> Self {
        const MILLIS_IN_DAY: u64 = SECS_IN_DAY * 1_000;
        Self {
            months: 0,
            days: (millis / MILLIS_IN_DAY) as i32,
            micros: ((millis % MILLIS_IN_DAY) * 1_000) as i64,
        }
    }

    pub const fn from_secs(secs: u64) -> Self {
        Self {
            months: 0,
            days: (secs / SECS_IN_DAY) as i32,
            micros: ((secs % SECS_IN_DAY) * 1_000_000) as i64,
        }
    }

    pub const fn from_mins(mins: u64) -> Self {
        const MINS_IN_DAYS: u64 = 60 * 24;
        Self {
            months: 0,
            days: (mins / MINS_IN_DAYS) as i32,
            micros: ((mins % MINS_IN_DAYS) * 60 * MICROS_IN_SEC) as i64,
        }
    }

    pub const fn from_days(days: u64) -> Self {
        Self {
            months: 0,
            days: days as i32,
            micros: 0,
        }
    }

    pub const fn from_weeks(weeks: u64) -> Self {
        Self {
            months: 0,
            days: (weeks * 7) as i32,
            micros: 0,
        }
    }

    pub const fn is_zero(&self) -> bool {
        self.months == 0 && self.days == 0 && self.micros == 0
    }

    pub const fn as_duration(&self, days_in_month: f64) -> Duration {
        const NANOS_IN_SEC: i128 = 1_000_000_000;
        const NANOS_IN_DAY: i128 = SECS_IN_DAY as i128 * NANOS_IN_SEC;
        let nanos = (self.months as f64) * days_in_month * (NANOS_IN_DAY as f64); // months
        let nanos = nanos as i128 + self.days as i128 * NANOS_IN_DAY; // days
        let nanos = nanos + self.micros as i128 * 1000; // micros
        let secs = (nanos / NANOS_IN_SEC) as u64;
        let nanos = (nanos % NANOS_IN_SEC) as u32;
        Duration::new(secs, nanos)
    }
}
