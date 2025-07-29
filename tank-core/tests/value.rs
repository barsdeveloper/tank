#![feature(assert_matches)]

#[cfg(test)]
mod tests {
    use rust_decimal::{Decimal, prelude::FromPrimitive};
    use std::{assert_matches::assert_matches, borrow::Cow};
    use tank_core::{AsValue, Value};
    use time::Month;

    #[test]
    fn value_none() {
        assert_eq!(Value::Null, Value::Null);
        assert_ne!(Value::Float32(Some(1.0)), Value::Null);
    }

    #[test]
    fn value_bool() {
        let var = true;
        let val: Value = var.into();
        assert_eq!(val, Value::Boolean(Some(true)));
        assert_ne!(val, Value::Boolean(Some(false)));
        assert_ne!(val, Value::Boolean(None));
        assert_ne!(val, Value::Varchar(Some("true".into())));
        let var: bool = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: bool = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, true);
        assert_eq!(bool::try_from_value((1 as i8).into()).unwrap(), true);
        assert_eq!(bool::try_from_value((8 as i16).into()).unwrap(), true);
        assert_eq!(bool::try_from_value((0 as i32).into()).unwrap(), false);
        assert_eq!(bool::try_from_value((0 as i64).into()).unwrap(), false);
        assert_eq!(bool::try_from_value((9 as i128).into()).unwrap(), true);
        assert_eq!(bool::try_from_value((0 as u8).into()).unwrap(), false);
        assert_eq!(bool::try_from_value((1 as u16).into()).unwrap(), true);
        assert_eq!(bool::try_from_value((1 as u32).into()).unwrap(), true);
        assert_eq!(bool::try_from_value((0 as u64).into()).unwrap(), false);
        assert_eq!(bool::try_from_value((2 as u128).into()).unwrap(), true);
        assert_matches!(bool::try_from_value((0.5 as f32).into()), Err(..));
    }

    #[test]
    fn value_i8() {
        let var = 127 as i8;
        let val: Value = var.into();
        assert_eq!(val, Value::Int8(Some(127)));
        assert_ne!(val, Value::Int8(Some(126)));
        let var: i8 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: i8 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, 127);
        assert_eq!(i8::try_from_value((99 as u8).into()).unwrap(), 99);
        assert_matches!(i8::try_from_value((0.1 as f64).into()), Err(..));
    }

    #[test]
    fn value_i16() {
        let var = -32768 as i16;
        let val: Value = var.into();
        assert_eq!(val, Value::Int16(Some(-32768)));
        assert_ne!(val, Value::Int32(Some(-32768)));
        let var: i16 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: i16 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, -32768 as i16);
        assert_eq!(i16::try_from_value((29 as i8).into()).unwrap(), 29);
        assert_eq!(i16::try_from_value((100 as u8).into()).unwrap(), 100);
        assert_eq!(i16::try_from_value((5000 as u16).into()).unwrap(), 5000);
    }

    #[test]
    fn value_i32() {
        let var = -2147483648 as i32;
        let val: Value = var.into();
        assert_eq!(val, Value::Int32(Some(-2147483648)));
        assert_ne!(val, Value::Null);
        let var: i32 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: i32 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, -2147483648 as i32);
        assert_eq!(i32::try_from_value((-31 as i8).into()).unwrap(), -31);
        assert_eq!(i32::try_from_value((-1 as i16).into()).unwrap(), -1);
        assert_eq!(i32::try_from_value((77 as u8).into()).unwrap(), 77);
        assert_eq!(i32::try_from_value((15 as u16).into()).unwrap(), 15);
        assert_eq!(i32::try_from_value((1001 as u32).into()).unwrap(), 1001);
    }

    #[test]
    fn value_i64() {
        let var = 9223372036854775807 as i64;
        let val: Value = var.into();
        let var: i64 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: i64 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, 9223372036854775807 as i64);
        assert_eq!(i64::try_from_value((-31 as i8).into()).unwrap(), -31);
        assert_eq!(i64::try_from_value((-1234 as i16).into()).unwrap(), -1234);
        assert_eq!(i64::try_from_value((-1 as i32).into()).unwrap(), -1);
        assert_eq!(i64::try_from_value((77 as u8).into()).unwrap(), 77);
        assert_eq!(i64::try_from_value((5555 as u16).into()).unwrap(), 5555);
        assert_eq!(i64::try_from_value((123456 as u32).into()).unwrap(), 123456);
        assert_eq!(
            i64::try_from_value((12345678901234 as u64).into()).unwrap(),
            12345678901234
        );
    }

    #[test]
    fn value_i128() {
        let var = -123456789101112131415 as i128;
        let val: Value = var.into();
        let var: i128 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: i128 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, -123456789101112131415 as i128);
        assert_eq!(i128::try_from_value((-31 as i8).into()).unwrap(), -31);
        assert_eq!(i128::try_from_value((-1234 as i16).into()).unwrap(), -1234);
        assert_eq!(i128::try_from_value((-1 as i32).into()).unwrap(), -1);
        assert_eq!(
            i128::try_from_value((-12345678901234 as i64).into()).unwrap(),
            -12345678901234
        );
        assert_eq!(i128::try_from_value((77 as u8).into()).unwrap(), 77);
        assert_eq!(i128::try_from_value((5555 as u16).into()).unwrap(), 5555);
        assert_eq!(
            i128::try_from_value((123456 as u32).into()).unwrap(),
            123456
        );
        assert_eq!(
            i128::try_from_value((12345678901234 as u64).into()).unwrap(),
            12345678901234
        );
        assert_eq!(
            i128::try_from_value((170141183460469231731687303715884105727 as u128).into()).unwrap(),
            170141183460469231731687303715884105727
        );
    }

    #[test]
    fn value_u8() {
        let var = 255 as u8;
        let val: Value = var.into();
        let var: u8 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: u8 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, 255);
    }

    #[test]
    fn value_u16() {
        let var = 65535 as u16;
        let val: Value = var.into();
        let var: u16 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: u16 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, 65535);
        assert_eq!(u16::try_from_value((123 as u8).into()).unwrap(), 123);
    }

    #[test]
    fn value_u32() {
        let var = 4_000_000_000 as u32;
        let val: Value = var.into();
        let var: u32 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: u32 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, 4_000_000_000);
        assert_eq!(u32::try_from_value((12 as u8).into()).unwrap(), 12);
        assert_eq!(u32::try_from_value((65535 as u16).into()).unwrap(), 65535);
    }

    #[test]
    fn value_u64() {
        let var = 18_000_000_000_000_000_000 as u64;
        let val: Value = var.into();
        let var: u64 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: u64 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, 18_000_000_000_000_000_000);
        assert_eq!(u64::try_from_value((77 as u8).into()).unwrap(), 77);
        assert_eq!(u64::try_from_value((1234 as u16).into()).unwrap(), 1234);
        assert_eq!(u64::try_from_value((123456 as u32).into()).unwrap(), 123456);
    }

    #[test]
    fn value_u128() {
        let var = 340_282_366_920_938_463_463_374_607_431_768_211_455 as u128;
        let val: Value = var.into();
        let var: u128 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: u128 = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, 340_282_366_920_938_463_463_374_607_431_768_211_455);
        assert_eq!(u128::try_from_value((11 as u8).into()).unwrap(), 11);
        assert_eq!(u128::try_from_value((222 as u16).into()).unwrap(), 222);
        assert_eq!(
            u128::try_from_value((333_333 as u32).into()).unwrap(),
            333_333
        );
        assert_eq!(
            u128::try_from_value((444_444_444_444 as u64).into()).unwrap(),
            444_444_444_444
        );
    }

    #[test]
    fn value_f32() {
        let var = 3.14f32;
        let val: Value = var.into();
        let var: f32 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: f32 = AsValue::try_from_value(val).unwrap();
        assert!((var - 3.14).abs() < f32::EPSILON);
        assert_eq!(
            f32::try_from_value(Decimal::from_f64(2.125).into()).unwrap(),
            2.125
        );
    }

    #[test]
    fn value_f64() {
        let var = 2.7182818284f64;
        let val: Value = var.into();
        let var: f64 = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: f64 = AsValue::try_from_value(val).unwrap();
        assert!((var - 2.7182818284).abs() < f64::EPSILON);
        assert_eq!(f64::try_from_value((3.5 as f32).into()).unwrap(), 3.5);
        assert_eq!(
            f64::try_from_value(Decimal::from_f64(2.25).into()).unwrap(),
            2.25
        );
    }

    #[test]
    fn value_string() {
        let var = "Hello World!";
        let val: Value = var.into();
        assert_eq!(val, Value::Varchar(Some("Hello World!".into())));
        assert_ne!(val, Value::Varchar(Some("Hello World.".into())));
        let var: String = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: String = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, "Hello World!");
    }

    #[test]
    fn value_cow_str() {
        let var = Cow::Borrowed("Hello World!");
        let val: Value = var.into();
        assert_eq!(val, Value::Varchar(Some("Hello World!".into())));
        let var: Cow<'_, str> = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: Cow<'_, str> = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, "Hello World!");
        assert_matches!(
            <Cow<'static, str> as AsValue>::as_empty_value(),
            Value::Varchar(..)
        );
        assert_matches!(
            <Cow<'static, str> as AsValue>::try_from_value(Value::Boolean(Some(false))),
            Err(..),
        );
    }

    #[test]
    fn value_date() {
        let var = time::Date::from_calendar_date(2025, Month::July, 21).unwrap();
        let val: Value = var.into();
        assert_eq!(val, Value::Date(Some(var)));
        assert_ne!(val, Value::Null);
        let var: time::Date = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: time::Date = AsValue::try_from_value(val).unwrap();
        assert_eq!(
            var,
            time::Date::from_calendar_date(2025, Month::July, 21).unwrap()
        );
        let val: time::Date =
            AsValue::try_from_value(Value::Varchar(Some("2025-01-22".into()))).unwrap();
        assert_eq!(
            val,
            time::Date::from_calendar_date(2025, Month::January, 22).unwrap()
        );
    }

    #[test]
    fn value_time() {
        let var = time::Time::from_hms(0, 57, 21).unwrap();
        let val: Value = var.into();
        assert_eq!(val, Value::Time(Some(var)));
        assert_ne!(val, Value::Null);
        let var: time::Time = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: time::Time = AsValue::try_from_value(val).unwrap();
        assert_eq!(var, time::Time::from_hms(0, 57, 21).unwrap());
        assert_eq!(
            time::Time::try_from_value(Value::Varchar(Some("13:22".into()))).unwrap(),
            time::Time::from_hms(13, 22, 0).unwrap()
        );
    }

    #[test]
    fn value_datetime() {
        let var = time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2025, Month::July, 29).unwrap(),
            time::Time::from_hms(13, 52, 13).unwrap(),
        );
        let val: Value = var.into();
        assert_eq!(val, Value::Timestamp(Some(var)));
        assert_ne!(val, Value::Varchar(None));
        let var: time::PrimitiveDateTime = AsValue::try_from_value(val).unwrap();
        let val = var.as_value();
        let var: time::PrimitiveDateTime = AsValue::try_from_value(val).unwrap();
        assert_eq!(
            var,
            time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2025, Month::July, 29).unwrap(),
                time::Time::from_hms(13, 52, 13).unwrap(),
            )
        );
        let val: time::PrimitiveDateTime =
            AsValue::try_from_value(Value::Varchar(Some("2025-07-29T14:52:36.500".into())))
                .unwrap();
        assert_eq!(
            val,
            time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2025, Month::July, 29).unwrap(),
                time::Time::from_hms_milli(14, 52, 36, 500).unwrap()
            )
        );
        assert_ne!(
            val,
            time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2025, Month::July, 29).unwrap(),
                time::Time::from_hms(14, 52, 36).unwrap()
            )
        );
        let val: time::PrimitiveDateTime =
            AsValue::try_from_value(Value::Varchar(Some("2025-07-29T14:52:36".into()))).unwrap();
        assert_eq!(
            val,
            time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2025, Month::July, 29).unwrap(),
                time::Time::from_hms(14, 52, 36).unwrap()
            )
        );
        let val: time::PrimitiveDateTime =
            AsValue::try_from_value(Value::Varchar(Some("2025-07-29 14:52:36.500".into())))
                .unwrap();
        assert_eq!(
            val,
            time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2025, Month::July, 29).unwrap(),
                time::Time::from_hms_milli(14, 52, 36, 500).unwrap()
            )
        );
        let val: time::PrimitiveDateTime =
            AsValue::try_from_value(Value::Varchar(Some("2025-07-29 14:52:36".into()))).unwrap();
        assert_eq!(
            val,
            time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2025, Month::July, 29).unwrap(),
                time::Time::from_hms(14, 52, 36).unwrap()
            )
        );
        let val: time::PrimitiveDateTime =
            AsValue::try_from_value(Value::Varchar(Some("2025-07-29 14:52".into()))).unwrap();
        assert_eq!(
            val,
            time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2025, Month::July, 29).unwrap(),
                time::Time::from_hms(14, 52, 00).unwrap()
            )
        );
    }
}
