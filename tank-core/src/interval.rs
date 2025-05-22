use std::{hash::Hash, time::Duration};

const SECS_IN_DAY: i64 = 60 * 60 * 24;
const NANOS_IN_SEC: i128 = 1_000_000_000;
const NANOS_IN_DAY: i128 = SECS_IN_DAY as i128 * NANOS_IN_SEC;

#[derive(Default, Debug, Clone, Copy)]
pub struct Interval {
    months: i64,
    days: i64,
    nanos: i128,
}

impl Interval {
    pub const DAYS_IN_MONTH: f64 = 30.436875;

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
            nanos: duration.as_nanos() as i128,
        }
    }

    pub const fn from_nanos(nanos: i128) -> Self {
        Self {
            months: 0,
            days: (nanos / NANOS_IN_DAY) as _,
            nanos: (nanos % NANOS_IN_DAY),
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
            days,
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
        let nanos = (self.months as f64) * days_in_month * (NANOS_IN_DAY as f64); // months
        let nanos = nanos as i128 + self.days as i128 * NANOS_IN_DAY; // days
        let nanos = nanos + self.nanos as i128;
        let secs = (nanos / NANOS_IN_SEC) as u64;
        let nanos = (nanos % NANOS_IN_SEC) as u32;
        Duration::new(secs, nanos)
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.months == other.months
            && self.days as i128 * NANOS_IN_DAY + self.nanos
                == other.days as i128 * NANOS_IN_DAY + other.nanos
    }
}

impl Hash for Interval {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.months.hash(state);
        (self.days as i128 * NANOS_IN_DAY + self.nanos).hash(state);
    }
}
