use std::time::Duration;

const SECS_IN_DAY: i64 = 60 * 60 * 24;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    months: i64,
    days: i64,
    nanos: i128,
}

impl Interval {
    pub fn new(months: i64, days: i64, nanos: i128) -> Self {
        Self {
            months,
            days,
            nanos,
        }
    }

    pub const fn from_duration(duration: &Duration) -> Self {
        Self {
            months: 0,
            days: 0,
            nanos: duration.as_nanos() as _,
        }
    }

    pub const fn from_nanos(micros: i128) -> Self {
        const NANOS_IN_DAY: i128 = (SECS_IN_DAY * 1_000_000_000) as _;
        Self {
            months: 0,
            days: (micros / NANOS_IN_DAY) as _,
            nanos: (micros % NANOS_IN_DAY),
        }
    }

    pub const fn from_micros(micros: i128) -> Self {
        const MICROS_IN_DAY: i128 = (SECS_IN_DAY * 1_000_000) as _;
        Self {
            months: 0,
            days: (micros / MICROS_IN_DAY) as _,
            nanos: (micros % MICROS_IN_DAY) * 1_000_000,
        }
    }

    pub const fn from_millis(millis: i128) -> Self {
        const MILLIS_IN_DAY: i128 = (SECS_IN_DAY * 1_000) as _;
        Self {
            months: 0,
            days: (millis / MILLIS_IN_DAY) as _,
            nanos: ((millis % MILLIS_IN_DAY) * 1_000),
        }
    }

    pub const fn from_secs(secs: i64) -> Self {
        Self {
            months: 0,
            days: (secs / SECS_IN_DAY) as _,
            nanos: ((secs % SECS_IN_DAY) * 1_000_000_000) as _,
        }
    }

    pub const fn from_mins(mins: i64) -> Self {
        const MINS_IN_DAYS: i64 = 60 * 24;
        Self {
            months: 0,
            days: (mins / MINS_IN_DAYS),
            nanos: ((mins % MINS_IN_DAYS) * 60 * 1_000_000_000) as _,
        }
    }

    pub const fn from_days(days: i64) -> Self {
        Self {
            months: 0,
            days: days,
            nanos: 0,
        }
    }

    pub const fn from_weeks(weeks: i64) -> Self {
        Self {
            months: 0,
            days: weeks * 7,
            nanos: 0,
        }
    }

    pub const fn is_zero(&self) -> bool {
        self.months == 0 && self.days == 0 && self.nanos == 0
    }

    pub const fn as_duration(&self, days_in_month: f64) -> Duration {
        const NANOS_IN_SEC: i128 = 1_000_000_000;
        const NANOS_IN_DAY: i128 = SECS_IN_DAY as i128 * NANOS_IN_SEC;
        let nanos = (self.months as f64) * days_in_month * (NANOS_IN_DAY as f64); // months
        let nanos = nanos as i128 + self.days as i128 * NANOS_IN_DAY; // days
        let nanos = nanos + self.nanos as i128 * 1000; // micros
        let secs = (nanos / NANOS_IN_SEC) as u64;
        let nanos = (nanos % NANOS_IN_SEC) as u32;
        Duration::new(secs, nanos)
    }
}
