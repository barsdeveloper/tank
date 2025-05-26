use std::{
    hash::Hash,
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration,
};

#[derive(Default, Debug, Clone, Copy)]
pub struct Interval {
    pub months: i64,
    pub days: i64,
    pub nanos: i128,
}

impl Interval {
    pub const DAYS_IN_MONTH: f64 = 30.436875;
    pub const SECS_IN_DAY: i64 = 60 * 60 * 24;
    pub const NANOS_IN_SEC: i128 = 1_000_000_000;
    pub const NANOS_IN_DAY: i128 = Self::SECS_IN_DAY as i128 * Self::NANOS_IN_SEC;

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

    pub const fn from_nanos(value: i128) -> Self {
        Self {
            months: 0,
            days: (value / Self::NANOS_IN_DAY) as _,
            nanos: (value % Self::NANOS_IN_DAY),
        }
    }

    pub const fn from_micros(value: i128) -> Self {
        const MICROS_IN_DAY: i128 = (Interval::SECS_IN_DAY * 1_000_000) as _;
        Self {
            months: 0,
            days: (value / MICROS_IN_DAY) as _,
            nanos: (value % MICROS_IN_DAY) * 1_000,
        }
    }

    pub const fn from_millis(value: i128) -> Self {
        const MILLIS_IN_DAY: i128 = (Interval::SECS_IN_DAY * 1_000) as _;
        Self {
            months: 0,
            days: (value / MILLIS_IN_DAY) as _,
            nanos: ((value % MILLIS_IN_DAY) * 1_000_000),
        }
    }

    pub const fn from_secs(value: i64) -> Self {
        Self {
            months: 0,
            days: (value / Self::SECS_IN_DAY) as _,
            nanos: ((value % Self::SECS_IN_DAY) * 1_000_000_000) as _,
        }
    }

    pub const fn from_mins(value: i64) -> Self {
        const MINS_IN_DAYS: i64 = 60 * 24;
        Self {
            months: 0,
            days: (value / MINS_IN_DAYS),
            nanos: ((value % MINS_IN_DAYS) * 60 * 1_000_000_000) as _,
        }
    }

    pub const fn from_days(value: i64) -> Self {
        Self {
            months: 0,
            days: value,
            nanos: 0,
        }
    }

    pub const fn from_weeks(value: i64) -> Self {
        Self {
            months: 0,
            days: value * 7,
            nanos: 0,
        }
    }

    pub const fn from_months(value: i64) -> Self {
        Self {
            months: value,
            days: 0,
            nanos: 0,
        }
    }

    pub const fn is_zero(&self) -> bool {
        self.months == 0 && self.days == 0 && self.nanos == 0
    }

    pub const fn as_duration(&self, days_in_month: f64) -> Duration {
        let nanos = (self.months as f64) * days_in_month * (Interval::NANOS_IN_DAY as f64); // months
        let nanos = nanos as i128 + self.days as i128 * Interval::NANOS_IN_DAY; // days
        let nanos = nanos + self.nanos as i128;
        let secs = (nanos / Interval::NANOS_IN_SEC) as u64;
        let nanos = (nanos % Interval::NANOS_IN_SEC) as u32;
        Duration::new(secs, nanos)
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.months == other.months
            && self.days as i128 * Interval::NANOS_IN_DAY + self.nanos
                == other.days as i128 * Interval::NANOS_IN_DAY + other.nanos
    }
}

impl Eq for Interval {}

impl Hash for Interval {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.months.hash(state);
        (self.days as i128 * Interval::NANOS_IN_DAY + self.nanos).hash(state);
    }
}

macro_rules! sum_intervals {
    ($lhs:ident $op:tt $rhs:ident) => {{
        let days_total = $lhs.days as i128 $op $rhs.days as i128
            + $lhs.nanos / Interval::NANOS_IN_DAY $op $rhs.nanos / Interval::NANOS_IN_DAY;
        let days = days_total.clamp(i64::MIN as _, i64::MAX as _);
        let mut nanos = $lhs.nanos % Interval::NANOS_IN_DAY + $rhs.nanos % Interval::NANOS_IN_DAY;
        if days != days_total {
            nanos += (days_total - days) * Interval::NANOS_IN_DAY;
        }
        Interval {
            months: $lhs.months $op $rhs.months,
            days: days as _,
            nanos,
        }
    }};
}

impl Add for Interval {
    type Output = Interval;

    fn add(self, rhs: Self) -> Self {
        sum_intervals!(self + rhs)
    }
}

impl AddAssign for Interval {
    fn add_assign(&mut self, rhs: Self) {
        *self = sum_intervals!(self + rhs);
    }
}

impl Sub for Interval {
    type Output = Interval;

    fn sub(self, rhs: Self) -> Self::Output {
        sum_intervals!(self - rhs)
    }
}

impl SubAssign for Interval {
    fn sub_assign(&mut self, rhs: Self) {
        *self = sum_intervals!(self - rhs);
    }
}
