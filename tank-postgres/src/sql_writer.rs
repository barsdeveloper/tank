use std::fmt::Write;
use tank_core::{Context, SqlWriter, Value, future::Either, separated_by};
use time::{Date, PrimitiveDateTime, Time};

pub struct PostgresSqlWriter {}

impl SqlWriter for PostgresSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_column_type(&self, context: &mut Context, buff: &mut String, value: &Value) {
        match value {
            Value::Boolean(..) => buff.push_str("BOOLEAN"),
            Value::Int8(..) => buff.push_str("SMALLINT"),
            Value::Int16(..) => buff.push_str("SMALLINT"),
            Value::Int32(..) => buff.push_str("INTEGER"),
            Value::Int64(..) => buff.push_str("BIGINT"),
            Value::Int128(..) => buff.push_str("NUMERIC(38)"),
            Value::UInt8(..) => buff.push_str("SMALLINT"),
            Value::UInt16(..) => buff.push_str("INTEGER"),
            Value::UInt32(..) => buff.push_str("BIGINT"),
            Value::UInt64(..) => buff.push_str("NUMERIC(19)"),
            Value::UInt128(..) => buff.push_str("NUMERIC(38)"),
            Value::Float32(..) => buff.push_str("FLOAT4"),
            Value::Float64(..) => buff.push_str("FLOAT8"),
            Value::Decimal(.., precision, scale) => {
                buff.push_str("NUMERIC");
                if (precision, scale) != (&0, &0) {
                    let _ = write!(buff, "({},{})", precision, scale);
                }
            }
            Value::Char(..) => buff.push_str("CHARACTER(1)"),
            Value::Varchar(..) => buff.push_str("TEXT"),
            Value::Blob(..) => buff.push_str("BYTEA"),
            Value::Date(..) => buff.push_str("DATE"),
            Value::Time(..) => buff.push_str("TIME"),
            Value::Timestamp(..) => buff.push_str("TIMESTAMP"),
            Value::TimestampWithTimezone(..) => buff.push_str("TIMESTAMP WITH TIME ZONE"),
            Value::Interval(..) => buff.push_str("INTERVAL"),
            Value::Uuid(..) => buff.push_str("UUID"),
            Value::Array(.., inner, size) => {
                self.write_column_type(context, buff, inner);
                let _ = write!(buff, "[{}]", size);
            }
            Value::List(.., inner) => {
                self.write_column_type(context, buff, inner);
                buff.push_str("[]");
            }
            _ => log::error!(
                "Unexpected tank::Value, variant {:?} is not supported",
                value
            ),
        };
    }

    fn write_value_blob(&self, _context: &mut Context, buff: &mut String, value: &[u8]) {
        buff.push_str("'\\x");
        for b in value {
            let _ = write!(buff, "{:X}", b);
        }
        buff.push('\'');
    }

    fn write_value_date(
        &self,
        _context: &mut Context,
        buff: &mut String,
        value: &Date,
        timestamp: bool,
    ) {
        let (l, r) = if timestamp {
            ("", "")
        } else {
            ("'", "'::DATE")
        };
        let (year, suffix) = if !timestamp && value.year() <= 0 {
            // Year 0 in Postgres is 1 BC
            (value.year().abs() + 1, " BC")
        } else {
            (value.year(), "")
        };
        let _ = write!(
            buff,
            "{l}{:04}-{:02}-{:02}{suffix}{r}",
            year,
            value.month() as u8,
            value.day()
        );
    }

    fn write_value_time(
        &self,
        _context: &mut Context,
        buff: &mut String,
        value: &Time,
        timestamp: bool,
    ) {
        let mut subsecond = value.nanosecond();
        let mut width = 9;
        while width > 1 && subsecond % 10 == 0 {
            subsecond /= 10;
            width -= 1;
        }
        let (l, r) = if timestamp {
            ("", "")
        } else {
            ("'", "'::TIME")
        };
        let _ = write!(
            buff,
            "{l}{:02}:{:02}:{:02}.{:0width$}{r}",
            value.hour(),
            value.minute(),
            value.second(),
            subsecond
        );
    }

    fn write_value_timestamp(
        &self,
        context: &mut Context,
        buff: &mut String,
        value: &PrimitiveDateTime,
    ) {
        buff.push('\'');
        self.write_value_date(context, buff, &value.date(), true);
        buff.push('T');
        self.write_value_time(context, buff, &value.time(), true);
        if value.date().year() <= 0 {
            buff.push_str(" BC");
        }
        buff.push_str("'::TIMESTAMP");
    }

    fn write_value_list<'a>(
        &self,
        context: &mut Context,
        buff: &mut String,
        value: Either<&Box<[Value]>, &Vec<Value>>,
        ty: &Value,
    ) {
        buff.push_str("ARRAY[");
        separated_by(
            buff,
            match value {
                Either::Left(v) => v.iter(),
                Either::Right(v) => v.iter(),
            },
            |buff, v| {
                self.write_value(context, buff, v);
            },
            ",",
        );
        buff.push_str("]::");
        self.write_column_type(context, buff, ty);
    }

    fn write_expression_operand_question_mark(&self, context: &mut Context, buff: &mut String) {
        context.counter += 1;
        let _ = write!(buff, "${}", context.counter);
    }
}
