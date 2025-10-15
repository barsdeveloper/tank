use anyhow::Context;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, macros::format_description};

use crate::{Error, Result};
use std::cmp;

pub trait Parse {
    fn parse(value: impl AsRef<str>) -> Result<Self>
    where
        Self: Sized;
}

impl Parse for Box<[u8]> {
    fn parse(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        Err(Error::msg(format!(
            "Could not decode blob pattern: `{}{}`",
            &value[..cmp::min(value.len(), 497)],
            if value.len() > 497 { "..." } else { "" }
        )))
    }
}

impl Parse for Date {
    fn parse(value: impl AsRef<str>) -> Result<Self>
    where
        Self: Sized,
    {
        time::Date::parse(value.as_ref(), format_description!("[year]-[month]-[day]"))
            .with_context(|| format!("Cannot parse '{}' as time::Time", value.as_ref()))
    }
}

impl Parse for Time {
    fn parse(value: impl AsRef<str>) -> Result<Self>
    where
        Self: Sized,
    {
        let value = value.as_ref();
        time::Time::parse(
            value,
            format_description!("[hour]:[minute]:[second].[subsecond]"),
        )
        .or(time::Time::parse(
            value,
            format_description!("[hour]:[minute]:[second]"),
        ))
        .or(time::Time::parse(
            value,
            format_description!("[hour]:[minute]"),
        ))
        .with_context(|| format!("Cannot parse '{}' as time::Time", value))
    }
}

impl Parse for PrimitiveDateTime {
    fn parse(value: impl AsRef<str>) -> Result<Self>
    where
        Self: Sized,
    {
        let value = value.as_ref();
        time::PrimitiveDateTime::parse(
            value,
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]"),
        )
        .or(time::PrimitiveDateTime::parse(
            value,
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
        ))
        .or(time::PrimitiveDateTime::parse(
            value,
            format_description!("[year]-[month]-[day]T[hour]:[minute]"),
        ))
        .or(time::PrimitiveDateTime::parse(
            value,
            format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"),
        ))
        .or(time::PrimitiveDateTime::parse(
            value,
            format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        ))
        .or(time::PrimitiveDateTime::parse(
            value,
            format_description!("[year]-[month]-[day] [hour]:[minute]"),
        ))
        .with_context(|| format!("Cannot parse '{}' as time::PrimitiveDateTime", value))
    }
}

impl Parse for OffsetDateTime {
    fn parse(value: impl AsRef<str>) -> Result<Self>
    where
        Self: Sized,
    {
        let value = value.as_ref();
        time::OffsetDateTime::parse(
            value,
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]")
        )
        .or(time::OffsetDateTime::parse(
            value,
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]")
        ))
        .or(time::OffsetDateTime::parse(
            value,
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]")
        ))
        .or(time::OffsetDateTime::parse(
            value,
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]")
        ))
        .or(time::OffsetDateTime::parse(
            value,
            format_description!("[year]-[month]-[day]T[hour]:[minute][offset_hour sign:mandatory]:[offset_minute]")
        ))
        .or(time::OffsetDateTime::parse(
            value,
            format_description!("[year]-[month]-[day]T[hour]:[minute][offset_hour sign:mandatory]")
        ))
        .with_context(|| format!("Cannot parse '{}' as time::OffsetDateTime", value))
    }
}
