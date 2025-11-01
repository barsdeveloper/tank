use std::fmt::Write;
use tank_core::{Context, SqlWriter, Value, future::Either, separated_by};
use time::{Date, PrimitiveDateTime, Time};

pub struct PostgresSqlWriter {}

impl SqlWriter for PostgresSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_column_type(&self, context: &mut Context, out: &mut String, value: &Value) {
        match value {
            Value::Boolean(..) => out.push_str("BOOLEAN"),
            Value::Int8(..) => out.push_str("SMALLINT"),
            Value::Int16(..) => out.push_str("SMALLINT"),
            Value::Int32(..) => out.push_str("INTEGER"),
            Value::Int64(..) => out.push_str("BIGINT"),
            Value::Int128(..) => out.push_str("NUMERIC(38)"),
            Value::UInt8(..) => out.push_str("SMALLINT"),
            Value::UInt16(..) => out.push_str("INTEGER"),
            Value::UInt32(..) => out.push_str("BIGINT"),
            Value::UInt64(..) => out.push_str("NUMERIC(19)"),
            Value::UInt128(..) => out.push_str("NUMERIC(38)"),
            Value::Float32(..) => out.push_str("FLOAT4"),
            Value::Float64(..) => out.push_str("FLOAT8"),
            Value::Decimal(.., precision, scale) => {
                out.push_str("NUMERIC");
                if (precision, scale) != (&0, &0) {
                    let _ = write!(out, "({},{})", precision, scale);
                }
            }
            Value::Char(..) => out.push_str("CHAR(1)"),
            Value::Varchar(..) => out.push_str("TEXT"),
            Value::Blob(..) => out.push_str("BYTEA"),
            Value::Date(..) => out.push_str("DATE"),
            Value::Time(..) => out.push_str("TIME"),
            Value::Timestamp(..) => out.push_str("TIMESTAMP"),
            Value::TimestampWithTimezone(..) => out.push_str("TIMESTAMP WITH TIME ZONE"),
            Value::Interval(..) => out.push_str("INTERVAL"),
            Value::Uuid(..) => out.push_str("UUID"),
            Value::Array(.., inner, size) => {
                self.write_column_type(context, out, inner);
                let _ = write!(out, "[{}]", size);
            }
            Value::List(.., inner) => {
                self.write_column_type(context, out, inner);
                out.push_str("[]");
            }
            _ => log::error!(
                "Unexpected tank::Value, variant {:?} is not supported",
                value
            ),
        };
    }

    fn write_value_blob(&self, _context: &mut Context, out: &mut String, value: &[u8]) {
        out.push_str("'\\x");
        for b in value {
            let _ = write!(out, "{:X}", b);
        }
        out.push('\'');
    }

    fn write_value_date(
        &self,
        _context: &mut Context,
        out: &mut String,
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
            out,
            "{l}{:04}-{:02}-{:02}{suffix}{r}",
            year,
            value.month() as u8,
            value.day()
        );
    }

    fn write_value_time(
        &self,
        _context: &mut Context,
        out: &mut String,
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
            out,
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
        out: &mut String,
        value: &PrimitiveDateTime,
    ) {
        out.push('\'');
        self.write_value_date(context, out, &value.date(), true);
        out.push('T');
        self.write_value_time(context, out, &value.time(), true);
        if value.date().year() <= 0 {
            out.push_str(" BC");
        }
        out.push_str("'::TIMESTAMP");
    }

    fn write_value_list<'a>(
        &self,
        context: &mut Context,
        out: &mut String,
        value: Either<&Box<[Value]>, &Vec<Value>>,
        ty: &Value,
    ) {
        out.push_str("ARRAY[");
        separated_by(
            out,
            match value {
                Either::Left(v) => v.iter(),
                Either::Right(v) => v.iter(),
            },
            |out, v| {
                self.write_value(context, out, v);
            },
            ",",
        );
        out.push_str("]::");
        self.write_column_type(context, out, ty);
    }

    fn write_expression_operand_question_mark(&self, context: &mut Context, out: &mut String) {
        context.counter += 1;
        let _ = write!(out, "${}", context.counter);
    }
}
