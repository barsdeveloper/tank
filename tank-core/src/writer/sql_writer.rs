use crate::{
    BinaryOp, BinaryOpType, ColumnDef, ColumnRef, DataSet, EitherIterator, Entity, Expression,
    Fragment, Interval, Join, JoinType, Operand, PrimaryKeyType, TableRef, UnaryOp, UnaryOpType,
    Value, possibly_parenthesized, separated_by, writer::Context,
};
use std::{collections::HashMap, fmt::Write};
use time::{Date, Time};

macro_rules! write_integer {
    ($out:ident, $value:expr) => {{
        let mut buffer = itoa::Buffer::new();
        let _ = $out.write_str(buffer.format($value));
    }};
}
macro_rules! write_float {
    ($this:ident, $context:ident,$out:ident, $value:expr) => {{
        if $value.is_infinite() {
            $this.write_value_infinity($context, $out, $value.is_sign_negative());
        } else {
            let mut buffer = ryu::Buffer::new();
            let _ = $out.write_str(buffer.format($value));
        }
    }};
}

pub trait SqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter;

    fn alias_declaration(&self, context: Context) -> bool {
        match context.fragment {
            Fragment::SqlSelectFrom | Fragment::SqlJoin => true,
            _ => false,
        }
    }

    fn write_escaped(
        &self,
        _: Context,
        out: &mut dyn Write,
        value: &str,
        search: char,
        replace: &str,
    ) {
        let mut position = 0;
        for (i, c) in value.char_indices() {
            if c == search {
                let _ = out.write_str(&value[position..i]);
                let _ = out.write_str(replace);
                position = i + 1;
            }
        }
        let _ = out.write_str(&value[position..]);
    }

    fn write_identifier_quoted(&self, context: Context, out: &mut dyn Write, value: &str) {
        let _ = out.write_char('"');
        self.write_escaped(context, out, value, '"', r#""""#);
        let _ = out.write_char('"');
    }

    fn write_table_ref(&self, context: Context, out: &mut dyn Write, value: &TableRef) {
        if self.alias_declaration(context) || value.alias.is_empty() {
            if !value.schema.is_empty() {
                self.write_identifier_quoted(context, out, &value.schema);
                let _ = out.write_char('.');
            }
            self.write_identifier_quoted(context, out, &value.name);
        }
        if !value.alias.is_empty() {
            let _ = write!(out, " {}", value.alias);
        }
    }

    fn write_column_ref(&self, context: Context, out: &mut dyn Write, value: &ColumnRef) {
        if context.qualify_columns && !value.table.is_empty() {
            if !value.schema.is_empty() {
                self.write_identifier_quoted(context, out, &value.schema);
                let _ = out.write_char('.');
            }
            self.write_identifier_quoted(context, out, &value.table);
            let _ = out.write_char('.');
        }
        self.write_identifier_quoted(context, out, &value.name);
    }

    fn write_column_type(&self, context: Context, out: &mut dyn Write, value: &Value) {
        let _ = match value {
            Value::Boolean(..) => out.write_str("BOOLEAN"),
            Value::Int8(..) => out.write_str("TINYINT"),
            Value::Int16(..) => out.write_str("SMALLINT"),
            Value::Int32(..) => out.write_str("INTEGER"),
            Value::Int64(..) => out.write_str("BIGINT"),
            Value::Int128(..) => out.write_str("HUGEINT"),
            Value::UInt8(..) => out.write_str("UTINYINT"),
            Value::UInt16(..) => out.write_str("USMALLINT"),
            Value::UInt32(..) => out.write_str("UINTEGER"),
            Value::UInt64(..) => out.write_str("UBIGINT"),
            Value::UInt128(..) => out.write_str("UHUGEINT"),
            Value::Float32(..) => out.write_str("FLOAT"),
            Value::Float64(..) => out.write_str("DOUBLE"),
            Value::Decimal(.., precision, scale) => {
                let _ = out.write_str("DECIMAL");
                if (precision, scale) != (&0, &0) {
                    write!(out, "({},{})", precision, scale)
                } else {
                    Ok(())
                }
            }
            Value::Char(..) => out.write_str("CHAR(1)"),
            Value::Varchar(..) => out.write_str("VARCHAR"),
            Value::Blob(..) => out.write_str("BLOB"),
            Value::Date(..) => out.write_str("DATE"),
            Value::Time(..) => out.write_str("TIME"),
            Value::Timestamp(..) => out.write_str("TIMESTAMP"),
            Value::TimestampWithTimezone(..) => out.write_str("TIMESTAMP WITH TIME ZONE"),
            Value::Interval(..) => out.write_str("INTERVAL"),
            Value::Uuid(..) => out.write_str("UUID"),
            Value::Array(.., inner, size) => {
                self.write_column_type(context, out, inner);
                write!(out, "[{}]", size)
            }
            Value::List(.., inner) => {
                self.write_column_type(context, out, inner);
                out.write_str("[]")
            }
            Value::Map(.., key, value) => {
                let _ = out.write_str("MAP(");
                self.write_column_type(context, out, key);
                let _ = out.write_char(',');
                self.write_column_type(context, out, value);
                let _ = out.write_char(')');
                Ok(())
            }
            _ => panic!(
                "Unexpected tank::Value, cannot get the sql type from {:?} variant",
                value
            ),
        };
    }

    fn write_value(&self, context: Context, out: &mut dyn Write, value: &Value) {
        let _ = match value {
            Value::Null
            | Value::Boolean(None, ..)
            | Value::Int8(None, ..)
            | Value::Int16(None, ..)
            | Value::Int32(None, ..)
            | Value::Int64(None, ..)
            | Value::Int128(None, ..)
            | Value::UInt8(None, ..)
            | Value::UInt16(None, ..)
            | Value::UInt32(None, ..)
            | Value::UInt64(None, ..)
            | Value::UInt128(None, ..)
            | Value::Float32(None, ..)
            | Value::Float64(None, ..)
            | Value::Decimal(None, ..)
            | Value::Char(None, ..)
            | Value::Varchar(None, ..)
            | Value::Blob(None, ..)
            | Value::Date(None, ..)
            | Value::Time(None, ..)
            | Value::Timestamp(None, ..)
            | Value::TimestampWithTimezone(None, ..)
            | Value::Interval(None, ..)
            | Value::Uuid(None, ..)
            | Value::Array(None, ..)
            | Value::List(None, ..)
            | Value::Map(None, ..)
            | Value::Struct(None, ..) => self.write_value_none(context, out),
            Value::Boolean(Some(v), ..) => self.write_value_bool(context, out, *v),
            Value::Int8(Some(v), ..) => write_integer!(out, *v),
            Value::Int16(Some(v), ..) => write_integer!(out, *v),
            Value::Int32(Some(v), ..) => write_integer!(out, *v),
            Value::Int64(Some(v), ..) => write_integer!(out, *v),
            Value::Int128(Some(v), ..) => write_integer!(out, *v),
            Value::UInt8(Some(v), ..) => write_integer!(out, *v),
            Value::UInt16(Some(v), ..) => write_integer!(out, *v),
            Value::UInt32(Some(v), ..) => write_integer!(out, *v),
            Value::UInt64(Some(v), ..) => write_integer!(out, *v),
            Value::UInt128(Some(v), ..) => write_integer!(out, *v),
            Value::Float32(Some(v), ..) => write_float!(self, context, out, *v),
            Value::Float64(Some(v), ..) => write_float!(self, context, out, *v),
            Value::Decimal(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Char(Some(v), ..) => {
                let _ = out.write_char('\'');
                let _ = out.write_char(*v);
                let _ = out.write_char('\'');
            }
            Value::Varchar(Some(v), ..) => self.write_value_string(context, out, v),
            Value::Blob(Some(v), ..) => self.write_value_blob(context, out, v.as_ref()),
            Value::Date(Some(v), ..) => {
                let _ = out.write_char('\'');
                self.write_value_date(context, out, v);
                let _ = out.write_char('\'');
            }
            Value::Time(Some(v), ..) => {
                let _ = out.write_char('\'');
                self.write_value_time(context, out, v);
                let _ = out.write_char('\'');
            }
            Value::Timestamp(Some(v), ..) => {
                let _ = out.write_char('\'');
                self.write_value_date(context, out, &v.date());
                let _ = out.write_char('T');
                self.write_value_time(context, out, &v.time());
                let _ = out.write_char('\'');
            }
            Value::TimestampWithTimezone(Some(v), ..) => {
                let _ = out.write_char('\'');
                self.write_value_date(context, out, &v.date());
                let _ = out.write_char('T');
                self.write_value_time(context, out, &v.time());
                let _ = write!(
                    out,
                    "{:+02}:{:02}",
                    v.offset().whole_hours(),
                    v.offset().whole_minutes()
                );
                let _ = out.write_char('\'');
            }
            Value::Interval(Some(v), ..) => self.write_value_interval(context, out, v),
            Value::Uuid(Some(v), ..) => drop(write!(out, "'{}'", v)),
            Value::List(Some(..), ..) | Value::Array(Some(..), ..) => {
                let v = match value {
                    Value::List(Some(v), ..) => v.iter(),
                    Value::Array(Some(v), ..) => v.iter(),
                    _ => unreachable!(),
                };
                let _ = out.write_char('[');
                separated_by(
                    out,
                    v,
                    |out, v| {
                        self.write_value(context, out, v);
                        true
                    },
                    ",",
                );
                let _ = out.write_char(']');
            }
            Value::Map(Some(v), ..) => self.write_value_map(context, out, v),
            Value::Struct(Some(_v), ..) => {
                todo!()
            }
        };
    }

    fn write_value_none(&self, _context: Context, out: &mut dyn Write) {
        let _ = out.write_str("NULL");
    }

    fn write_value_bool(&self, _context: Context, out: &mut dyn Write, value: bool) {
        let _ = out.write_str(["false", "true"][value as usize]);
    }

    fn write_value_infinity(&self, context: Context, out: &mut dyn Write, negative: bool) {
        let mut buffer = ryu::Buffer::new();
        self.write_expression_binary_op(
            context,
            out,
            &BinaryOp {
                op: BinaryOpType::Cast,
                lhs: &Operand::LitStr(buffer.format(if negative {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                })),
                rhs: &Operand::Type(Value::Float64(None)),
            },
        );
    }

    fn write_value_string(&self, _context: Context, out: &mut dyn Write, value: &str) {
        let _ = out.write_char('\'');
        let mut position = 0;
        for (i, c) in value.char_indices() {
            if c == '\'' {
                let _ = out.write_str(&value[position..i]);
                let _ = out.write_str("''");
                position = i + 1;
            } else if c == '\n' {
                let _ = out.write_str(&value[position..i]);
                let _ = out.write_str("\\n");
                position = i + 1;
            }
        }
        let _ = out.write_str(&value[position..]);
        let _ = out.write_char('\'');
    }

    fn write_value_blob(&self, _context: Context, out: &mut dyn Write, value: &[u8]) {
        let _ = out.write_char('\'');
        for b in value {
            let _ = write!(out, "\\x{:X}", b);
        }
        let _ = out.write_char('\'');
    }

    fn write_value_date(&self, _context: Context, out: &mut dyn Write, value: &Date) {
        let _ = write!(
            out,
            "{:04}-{:02}-{:02}",
            value.year(),
            value.month() as u8,
            value.day()
        );
    }

    fn write_value_time(&self, _context: Context, out: &mut dyn Write, value: &Time) {
        let mut subsecond = value.nanosecond();
        let mut width = 9;
        while width > 1 && subsecond % 10 == 0 {
            subsecond /= 10;
            width -= 1;
        }
        let _ = write!(
            out,
            "{:02}:{:02}:{:02}.{:0width$}",
            value.hour(),
            value.minute(),
            value.second(),
            subsecond
        );
    }

    fn value_interval_units(&self) -> &[(&str, i128)] {
        static UNITS: &[(&str, i128)] = &[
            ("DAY", Interval::NANOS_IN_DAY),
            ("HOUR", Interval::NANOS_IN_SEC * 3600),
            ("MINUTE", Interval::NANOS_IN_SEC * 60),
            ("SECOND", Interval::NANOS_IN_SEC),
            ("MICROSECOND", 1_000),
            ("NANOSECOND", 1),
        ];
        UNITS
    }

    fn write_value_interval(&self, _context: Context, out: &mut dyn Write, value: &Interval) {
        let _ = out.write_str("INTERVAL ");
        macro_rules! write_unit {
            ($out:ident, $val:expr, $unit:expr) => {
                let _ = write!(
                    $out,
                    "{} {}{}",
                    $val,
                    $unit,
                    if $val != 1 { "S" } else { "" }
                );
            };
        }
        let months = value.months;
        let nanos = value.nanos + value.days as i128 * Interval::NANOS_IN_DAY;
        let multiple_units = nanos != 0 && value.months != 0;
        if multiple_units {
            let _ = out.write_char('\'');
        }
        if months != 0 {
            if months % 12 == 0 {
                write_unit!(out, months / 12, "YEAR");
            } else {
                write_unit!(out, months, "MONTH");
            }
        }
        for &(name, factor) in self.value_interval_units() {
            if nanos % factor == 0 {
                let value = nanos / factor;
                if value != 0 {
                    if months != 0 {
                        let _ = out.write_char(' ');
                    }
                    write_unit!(out, value, name);
                    break;
                }
            }
        }
        if multiple_units {
            let _ = out.write_char('\'');
        }
    }

    fn write_value_map(
        &self,
        context: Context,
        out: &mut dyn Write,
        value: &HashMap<Value, Value>,
    ) {
        let _ = out.write_char('{');
        separated_by(
            out,
            value,
            |out, (k, v)| {
                self.write_value(context, out, k);
                let _ = out.write_char(':');
                self.write_value(context, out, v);
                true
            },
            ",",
        );
        let _ = out.write_char('}');
    }

    fn expression_unary_op_precedence<'a>(&self, value: &UnaryOpType) -> i32 {
        match value {
            UnaryOpType::Negative => 1250,
            UnaryOpType::Not => 250,
        }
    }

    fn expression_binary_op_precedence<'a>(&self, value: &BinaryOpType) -> i32 {
        match value {
            BinaryOpType::Or => 100,
            BinaryOpType::And => 200,
            BinaryOpType::Equal => 300,
            BinaryOpType::NotEqual => 300,
            BinaryOpType::Less => 300,
            BinaryOpType::Greater => 300,
            BinaryOpType::LessEqual => 300,
            BinaryOpType::GreaterEqual => 300,
            BinaryOpType::Is => 400,
            BinaryOpType::IsNot => 400,
            BinaryOpType::Like => 400,
            BinaryOpType::NotLike => 400,
            BinaryOpType::Regexp => 400,
            BinaryOpType::NotRegexp => 400,
            BinaryOpType::Glob => 400,
            BinaryOpType::NotGlob => 400,
            BinaryOpType::BitwiseOr => 500,
            BinaryOpType::BitwiseAnd => 600,
            BinaryOpType::ShiftLeft => 700,
            BinaryOpType::ShiftRight => 700,
            BinaryOpType::Subtraction => 800,
            BinaryOpType::Addition => 800,
            BinaryOpType::Multiplication => 900,
            BinaryOpType::Division => 900,
            BinaryOpType::Remainder => 900,
            BinaryOpType::Indexing => 1000,
            BinaryOpType::Cast => 1100,
            BinaryOpType::Alias => 1200,
        }
    }

    fn write_expression_operand(&self, context: Context, out: &mut dyn Write, value: &Operand) {
        let _ = match value {
            Operand::LitBool(v) => self.write_value_bool(context, out, *v),
            Operand::LitFloat(v) => write_float!(self, context, out, *v),
            Operand::LitIdent(v) => drop(out.write_str(v)),
            Operand::LitField(v) => separated_by(out, *v, |out, v| out.write_str(v).is_ok(), "."),
            Operand::LitInt(v) => write_integer!(out, *v),
            Operand::LitStr(v) => self.write_value_string(context, out, v),
            Operand::LitArray(v) => {
                let _ = out.write_char('[');
                separated_by(
                    out,
                    *v,
                    |out, v| {
                        v.write_query(self.as_dyn(), context, out);
                        true
                    },
                    ", ",
                );
                let _ = out.write_char(']');
            }
            Operand::Null => drop(out.write_str("NULL")),
            Operand::Type(v) => self.write_column_type(context, out, v),
            Operand::Variable(v) => self.write_value(context, out, v),
            Operand::Call(f, args) => {
                let _ = out.write_str(f);
                let _ = out.write_char('(');
                separated_by(
                    out,
                    *args,
                    |out, v| {
                        v.write_query(self.as_dyn(), context, out);
                        true
                    },
                    ",",
                );
                let _ = out.write_char(')');
            }
            Operand::Asterisk => drop(out.write_char('*')),
            Operand::QuestionMark => drop(out.write_char('?')),
        };
    }

    fn write_expression_unary_op(
        self: &Self,
        context: Context,
        out: &mut dyn Write,
        value: &UnaryOp<&dyn Expression>,
    ) {
        let _ = match value.op {
            UnaryOpType::Negative => out.write_char('-'),
            UnaryOpType::Not => out.write_str("NOT "),
        };
        possibly_parenthesized!(
            out,
            value.arg.precedence(self.as_dyn()) <= self.expression_unary_op_precedence(&value.op),
            value.arg.write_query(self.as_dyn(), context, out)
        );
    }

    fn write_expression_binary_op(
        &self,
        context: Context,
        out: &mut dyn Write,
        value: &BinaryOp<&dyn Expression, &dyn Expression>,
    ) {
        let (prefix, infix, suffix, lhs_parenthesized, rhs_parenthesized) = match value.op {
            BinaryOpType::Indexing => ("", "[", "]", false, true),
            BinaryOpType::Cast => ("CAST(", " AS ", ")", true, true),
            BinaryOpType::Multiplication => ("", " * ", "", false, false),
            BinaryOpType::Division => ("", " / ", "", false, false),
            BinaryOpType::Remainder => ("", " % ", "", false, false),
            BinaryOpType::Addition => ("", " + ", "", false, false),
            BinaryOpType::Subtraction => ("", " - ", "", false, false),
            BinaryOpType::ShiftLeft => ("", " << ", "", false, false),
            BinaryOpType::ShiftRight => ("", " >> ", "", false, false),
            BinaryOpType::BitwiseAnd => ("", " & ", "", false, false),
            BinaryOpType::BitwiseOr => ("", " | ", "", false, false),
            BinaryOpType::Is => ("", " Is ", "", false, false),
            BinaryOpType::IsNot => ("", " IS NOT ", "", false, false),
            BinaryOpType::Like => ("", " LIKE ", "", false, false),
            BinaryOpType::NotLike => ("", " NOT LIKE ", "", false, false),
            BinaryOpType::Regexp => ("", " REGEXP ", "", false, false),
            BinaryOpType::NotRegexp => ("", " NOT REGEXP ", "", false, false),
            BinaryOpType::Glob => ("", " GLOB ", "", false, false),
            BinaryOpType::NotGlob => ("", " NOT GLOB ", "", false, false),
            BinaryOpType::Equal => ("", " = ", "", false, false),
            BinaryOpType::NotEqual => ("", " != ", "", false, false),
            BinaryOpType::Less => ("", " < ", "", false, false),
            BinaryOpType::LessEqual => ("", " <= ", "", false, false),
            BinaryOpType::Greater => ("", " > ", "", false, false),
            BinaryOpType::GreaterEqual => ("", " >= ", "", false, false),
            BinaryOpType::And => ("", " AND ", "", false, false),
            BinaryOpType::Or => ("", " OR ", "", false, false),
            BinaryOpType::Alias => ("", " AS ", "", false, false),
        };
        let precedence = self.expression_binary_op_precedence(&value.op);
        let _ = out.write_str(prefix);
        possibly_parenthesized!(
            out,
            !lhs_parenthesized && value.lhs.precedence(self.as_dyn()) < precedence,
            value.lhs.write_query(self.as_dyn(), context, out)
        );
        let _ = out.write_str(infix);
        possibly_parenthesized!(
            out,
            !rhs_parenthesized && value.rhs.precedence(self.as_dyn()) <= precedence,
            value.rhs.write_query(self.as_dyn(), context, out)
        );
        let _ = out.write_str(suffix);
    }

    fn write_join_type(&self, _context: Context, out: &mut dyn Write, join_type: &JoinType) {
        let _ = out.write_str(match &join_type {
            JoinType::Default => "JOIN",
            JoinType::Inner => "INNER JOIN",
            JoinType::Outer => "OUTER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Cross => "CROSS",
            JoinType::Natural => "NATURAL JOIN",
        });
    }

    fn write_join(
        &self,
        _context: Context,
        out: &mut dyn Write,
        join: &Join<&dyn DataSet, &dyn DataSet, &dyn Expression>,
    ) {
        let context = Context {
            fragment: Fragment::SqlJoin,
            qualify_columns: true,
        };
        join.lhs.write_query(self.as_dyn(), context, out);
        let _ = out.write_char(' ');
        self.write_join_type(context, out, &join.join);
        let _ = out.write_char(' ');
        join.rhs.write_query(self.as_dyn(), context, out);
        if let Some(on) = &join.on {
            let _ = out.write_str(" ON ");
            let context = Context {
                qualify_columns: true,
                ..context
            };
            on.write_query(self.as_dyn(), context, out);
        }
    }

    fn write_create_schema<E>(&self, out: &mut dyn Write, if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        let _ = out.write_str("CREATE SCHEMA ");
        let context = Context {
            fragment: Fragment::SqlCreateSchema,
            qualify_columns: E::qualified_columns(),
        };
        if if_not_exists {
            let _ = out.write_str("IF NOT EXISTS ");
        }
        self.write_identifier_quoted(context, out, E::table_ref().schema);
        let _ = out.write_char(';');
    }

    fn write_drop_schema<E>(&self, out: &mut dyn Write, if_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        let _ = out.write_str("DROP SCHEMA ");
        let context = Context {
            fragment: Fragment::SqlDropSchema,
            qualify_columns: E::qualified_columns(),
        };
        if if_exists {
            let _ = out.write_str("IF EXISTS ");
        }
        self.write_identifier_quoted(context, out, E::table_ref().schema);
        let _ = out.write_char(';');
    }

    fn write_create_table<E>(&self, out: &mut dyn Write, if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        let _ = out.write_str("CREATE TABLE ");
        if if_not_exists {
            let _ = out.write_str("IF NOT EXISTS ");
        }
        let context = Context {
            fragment: Fragment::SqlCreateTable,
            qualify_columns: E::qualified_columns(),
        };
        self.write_table_ref(context, out, E::table_ref());
        let _ = out.write_str(" (\n");
        separated_by(
            out,
            E::columns(),
            |out, v| {
                self.write_create_table_column_fragment(context, out, v);
                true
            },
            ",\n",
        );
        let primary_key = E::primary_key_def();
        if primary_key.len() > 1 {
            let _ = out.write_str(",\nPRIMARY KEY (");
            separated_by(
                out,
                primary_key,
                |out, v| {
                    self.write_identifier_quoted(
                        context.with_context(Fragment::SqlCreateTablePrimaryKey),
                        out,
                        v.name(),
                    );
                    true
                },
                ", ",
            );
            let _ = out.write_char(')');
        }
        for unique in E::unique_defs() {
            if unique.len() > 1 {
                let _ = out.write_str(",\nUNIQUE (");
                separated_by(
                    out,
                    unique,
                    |out, v| {
                        self.write_identifier_quoted(
                            context.with_context(Fragment::SqlCreateTableUnique),
                            out,
                            v.name(),
                        );
                        true
                    },
                    ", ",
                );
                let _ = out.write_char(')');
            }
        }
        let _ = out.write_str("\n)");
        let _ = out.write_char(';');
        self.write_column_comments::<E>(context, out);
    }

    fn write_column_comments<E>(&self, _context: Context, out: &mut dyn Write)
    where
        Self: Sized,
        E: Entity,
    {
        let context = Context {
            fragment: Fragment::SqlCommentOnColumn,
            qualify_columns: true,
        };
        for c in E::columns().iter().filter(|c| !c.comment.is_empty()) {
            let _ = out.write_str("\nCOMMENT ON COLUMN ");
            self.write_column_ref(context, out, c.into());
            let _ = out.write_str(" IS ");
            self.write_value_string(context, out, c.comment);
            let _ = out.write_char(';');
        }
    }

    fn write_create_table_column_fragment(
        &self,
        context: Context,
        out: &mut dyn Write,
        column: &ColumnDef,
    ) {
        self.write_identifier_quoted(context, out, &column.name());
        let _ = out.write_char(' ');
        if !column.column_type.is_empty() {
            let _ = out.write_str(&column.column_type);
        } else {
            SqlWriter::write_column_type(self, context, out, &column.value);
        }
        if !column.nullable && column.primary_key == PrimaryKeyType::None {
            let _ = out.write_str(" NOT NULL");
        }
        if let Some(default) = &column.default {
            let _ = out.write_str(" DEFAULT ");
            default.write_query(self.as_dyn(), context, out);
        }
        if column.primary_key == PrimaryKeyType::PrimaryKey {
            // Composite primary key will be printed elsewhere
            let _ = out.write_str(" PRIMARY KEY");
        }
        if column.unique && column.primary_key != PrimaryKeyType::PrimaryKey {
            let _ = out.write_str(" UNIQUE");
        }
        if let Some(references) = column.references {
            let _ = out.write_str(" REFERENCES ");
            self.write_table_ref(context, out, &references.table_ref());
            let _ = out.write_char('(');
            self.write_column_ref(context, out, &references);
            let _ = out.write_char(')');
        }
    }

    fn write_drop_table<E: Entity>(&self, out: &mut dyn Write, if_exists: bool)
    where
        Self: Sized,
    {
        let _ = out.write_str("DROP TABLE ");
        let context = Context {
            fragment: Fragment::SqlDropTable,
            qualify_columns: E::qualified_columns(),
        };
        if if_exists {
            let _ = out.write_str("IF EXISTS ");
        }
        self.write_table_ref(context, out, E::table_ref());
        let _ = out.write_char(';');
    }

    fn write_select<Item, Cols, Data, Cond>(
        &self,
        out: &mut dyn Write,
        columns: Cols,
        from: &Data,
        condition: &Cond,
        limit: Option<u32>,
    ) where
        Self: Sized,
        Item: Expression,
        Cols: IntoIterator<Item = Item> + Clone,
        Data: DataSet,
        Cond: Expression,
    {
        let _ = out.write_str("SELECT ");
        let mut has_order_by = false;
        let context = Context {
            fragment: Fragment::SqlSelect,
            qualify_columns: Data::qualified_columns(),
        };
        separated_by(
            out,
            columns.clone(),
            |out, col| {
                col.write_query(self, context, out);
                has_order_by = has_order_by || col.is_ordered();
                true
            },
            ", ",
        );
        let _ = out.write_str("\nFROM ");
        from.write_query(self, context.with_context(Fragment::SqlSelectFrom), out);
        let _ = out.write_str("\nWHERE ");
        condition.write_query(self, context.with_context(Fragment::SqlSelectWhere), out);
        if has_order_by {
            let _ = out.write_str("\nORDER BY ");
            for col in columns.into_iter().filter(Expression::is_ordered) {
                col.write_query(self, context.with_context(Fragment::SqlSelectOrderBy), out);
            }
        }
        if let Some(limit) = limit {
            let _ = write!(out, "\nLIMIT {}", limit);
        }
        let _ = out.write_char(';');
    }

    fn write_insert<'b, E, It>(&self, out: &mut dyn Write, entities: It, update: bool)
    where
        Self: Sized,
        E: Entity + 'b,
        It: IntoIterator<Item = &'b E>,
    {
        let mut rows = entities.into_iter().map(Entity::row_filtered).peekable();
        let Some(mut row) = rows.next() else {
            return;
        };
        let _ = out.write_str("INSERT INTO ");
        let mut context = Context {
            fragment: Fragment::SqlInsertInto,
            qualify_columns: E::qualified_columns(),
        };
        self.write_table_ref(context, out, E::table_ref());
        let _ = out.write_str(" (");
        let columns = E::columns().iter();
        let single = rows.peek().is_none();
        if single {
            // Inserting a single row uses row_labeled to filter out Passive::NotSet columns
            separated_by(
                out,
                row.iter(),
                |out, v| {
                    self.write_identifier_quoted(context, out, v.0);
                    true
                },
                ", ",
            );
        } else {
            // Inserting more rows will list all columns, Passive::NotSet columns will result in DEFAULT value
            separated_by(
                out,
                columns.clone(),
                |out, v| {
                    self.write_identifier_quoted(context, out, v.name());
                    true
                },
                ", ",
            );
        };
        let _ = out.write_str(") VALUES\n");
        context.fragment = Fragment::SqlInsertIntoValues;
        let mut first_row = None;
        let mut separate = false;
        loop {
            if separate {
                let _ = out.write_str(",\n");
            }
            let _ = out.write_char('(');
            let mut fields = row.iter();
            let mut field = fields.next();
            separated_by(
                out,
                E::columns(),
                |out, col| {
                    if Some(col.name()) == field.map(|v| v.0) {
                        self.write_value(
                            context,
                            out,
                            field
                                .map(|v| &v.1)
                                .expect(&format!("Column {} does not have a value", col.name())),
                        );
                        field = fields.next();
                        true
                    } else if !single {
                        let _ = out.write_str("DEFAULT");
                        true
                    } else {
                        false
                    }
                },
                ", ",
            );
            let _ = out.write_char(')');
            separate = true;
            if first_row.is_none() {
                first_row = row.into();
            }
            if let Some(next) = rows.next() {
                row = next;
            } else {
                break;
            };
        }
        let first_row = first_row
            .expect("Should have at least one row")
            .into_iter()
            .map(|(v, _)| v);
        if update {
            self.write_insert_update_fragment::<E, _>(
                context,
                out,
                if single {
                    EitherIterator::Left(
                        // If there is only one row to insert then list only the columns that appear
                        columns.filter(|c| first_row.clone().find(|n| *n == c.name()).is_some()),
                    )
                } else {
                    EitherIterator::Right(columns)
                },
            );
        }
        let _ = out.write_char(';');
    }

    fn write_insert_update_fragment<'a, E, It>(
        &self,
        mut context: Context,
        out: &mut dyn Write,
        columns: It,
    ) where
        Self: Sized,
        E: Entity,
        It: Iterator<Item = &'a ColumnDef>,
    {
        let pk = E::primary_key_def();
        if pk.len() == 0 {
            return;
        }
        let _ = out.write_str("\nON CONFLICT");
        context.fragment = Fragment::SqlInsertIntoOnConflict;
        if pk.len() > 0 {
            let _ = out.write_str(" (");
            separated_by(
                out,
                pk,
                |out, v| {
                    self.write_identifier_quoted(context, out, v.name());
                    true
                },
                ", ",
            );
            let _ = out.write_char(')');
        }
        let _ = out.write_str(" DO UPDATE SET\n");
        separated_by(
            out,
            columns.filter(|c| c.primary_key == PrimaryKeyType::None),
            |out, v| {
                self.write_identifier_quoted(context, out, v.name());
                let _ = out.write_str(" = EXCLUDED.");
                self.write_identifier_quoted(context, out, v.name());
                true
            },
            ",\n",
        );
    }

    fn write_delete<E: Entity, Expr: Expression>(&self, out: &mut dyn Write, condition: &Expr)
    where
        Self: Sized,
    {
        let _ = out.write_str("DELETE FROM ");
        let context = Context {
            fragment: Fragment::SqlDeleteFrom,
            qualify_columns: E::qualified_columns(),
        };
        self.write_table_ref(context, out, E::table_ref());
        let _ = out.write_str("\nWHERE ");
        condition.write_query(
            self,
            context.with_context(Fragment::SqlDeleteFromWhere),
            out,
        );
        let _ = out.write_char(';');
    }
}

pub struct GenericSqlWriter;
impl GenericSqlWriter {
    pub fn new() -> Self {
        Self {}
    }
}
impl SqlWriter for GenericSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }
}
