use crate::{
    BinaryOp, BinaryOpType, ColumnDef, ColumnRef, DataSet, EitherIterator, Entity, Expression,
    Fragment, Interval, Join, JoinType, Operand, Order, PrimaryKeyType, TableRef, UnaryOp,
    UnaryOpType, Value, possibly_parenthesized, separated_by, writer::Context,
};
use std::{collections::HashMap, fmt::Write};
use time::{Date, Time};

macro_rules! write_integer {
    ($buff:ident, $value:expr) => {{
        let mut buffer = itoa::Buffer::new();
        let _ = $buff.write_str(buffer.format($value));
    }};
}
macro_rules! write_float {
    ($this:ident, $context:ident,$buff:ident, $value:expr) => {{
        if $value.is_infinite() {
            $this.write_value_infinity($context, $buff, $value.is_sign_negative());
        } else {
            let mut buffer = ryu::Buffer::new();
            let _ = $buff.write_str(buffer.format($value));
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
        buff: &mut dyn Write,
        value: &str,
        search: char,
        replace: &str,
    ) {
        let mut position = 0;
        for (i, c) in value.char_indices() {
            if c == search {
                let _ = buff.write_str(&value[position..i]);
                let _ = buff.write_str(replace);
                position = i + 1;
            }
        }
        let _ = buff.write_str(&value[position..]);
    }

    fn write_identifier_quoted(&self, context: Context, buff: &mut dyn Write, value: &str) {
        let _ = buff.write_char('"');
        self.write_escaped(context, buff, value, '"', r#""""#);
        let _ = buff.write_char('"');
    }

    fn write_table_ref(&self, context: Context, buff: &mut dyn Write, value: &TableRef) {
        if self.alias_declaration(context) || value.alias.is_empty() {
            if !value.schema.is_empty() {
                self.write_identifier_quoted(context, buff, &value.schema);
                let _ = buff.write_char('.');
            }
            self.write_identifier_quoted(context, buff, &value.name);
        }
        if !value.alias.is_empty() {
            let _ = write!(buff, " {}", value.alias);
        }
    }

    fn write_column_ref(&self, context: Context, buff: &mut dyn Write, value: &ColumnRef) {
        if context.qualify_columns && !value.table.is_empty() {
            if !value.schema.is_empty() {
                self.write_identifier_quoted(context, buff, &value.schema);
                let _ = buff.write_char('.');
            }
            self.write_identifier_quoted(context, buff, &value.table);
            let _ = buff.write_char('.');
        }
        self.write_identifier_quoted(context, buff, &value.name);
    }

    fn write_column_type(&self, context: Context, buff: &mut dyn Write, value: &Value) {
        let _ = match value {
            Value::Boolean(..) => buff.write_str("BOOLEAN"),
            Value::Int8(..) => buff.write_str("TINYINT"),
            Value::Int16(..) => buff.write_str("SMALLINT"),
            Value::Int32(..) => buff.write_str("INTEGER"),
            Value::Int64(..) => buff.write_str("BIGINT"),
            Value::Int128(..) => buff.write_str("HUGEINT"),
            Value::UInt8(..) => buff.write_str("UTINYINT"),
            Value::UInt16(..) => buff.write_str("USMALLINT"),
            Value::UInt32(..) => buff.write_str("UINTEGER"),
            Value::UInt64(..) => buff.write_str("UBIGINT"),
            Value::UInt128(..) => buff.write_str("UHUGEINT"),
            Value::Float32(..) => buff.write_str("FLOAT"),
            Value::Float64(..) => buff.write_str("DOUBLE"),
            Value::Decimal(.., precision, scale) => {
                let _ = buff.write_str("DECIMAL");
                if (precision, scale) != (&0, &0) {
                    write!(buff, "({},{})", precision, scale)
                } else {
                    Ok(())
                }
            }
            Value::Char(..) => buff.write_str("CHAR(1)"),
            Value::Varchar(..) => buff.write_str("VARCHAR"),
            Value::Blob(..) => buff.write_str("BLOB"),
            Value::Date(..) => buff.write_str("DATE"),
            Value::Time(..) => buff.write_str("TIME"),
            Value::Timestamp(..) => buff.write_str("TIMESTAMP"),
            Value::TimestampWithTimezone(..) => buff.write_str("TIMESTAMP WITH TIME ZONE"),
            Value::Interval(..) => buff.write_str("INTERVAL"),
            Value::Uuid(..) => buff.write_str("UUID"),
            Value::Array(.., inner, size) => {
                self.write_column_type(context, buff, inner);
                write!(buff, "[{}]", size)
            }
            Value::List(.., inner) => {
                self.write_column_type(context, buff, inner);
                buff.write_str("[]")
            }
            Value::Map(.., key, value) => {
                let _ = buff.write_str("MAP(");
                self.write_column_type(context, buff, key);
                let _ = buff.write_char(',');
                self.write_column_type(context, buff, value);
                let _ = buff.write_char(')');
                Ok(())
            }
            _ => panic!(
                "Unexpected tank::Value, cannot get the sql type from {:?} variant",
                value
            ),
        };
    }

    fn write_value(&self, context: Context, buff: &mut dyn Write, value: &Value) {
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
            | Value::Struct(None, ..) => self.write_value_none(context, buff),
            Value::Boolean(Some(v), ..) => self.write_value_bool(context, buff, *v),
            Value::Int8(Some(v), ..) => write_integer!(buff, *v),
            Value::Int16(Some(v), ..) => write_integer!(buff, *v),
            Value::Int32(Some(v), ..) => write_integer!(buff, *v),
            Value::Int64(Some(v), ..) => write_integer!(buff, *v),
            Value::Int128(Some(v), ..) => write_integer!(buff, *v),
            Value::UInt8(Some(v), ..) => write_integer!(buff, *v),
            Value::UInt16(Some(v), ..) => write_integer!(buff, *v),
            Value::UInt32(Some(v), ..) => write_integer!(buff, *v),
            Value::UInt64(Some(v), ..) => write_integer!(buff, *v),
            Value::UInt128(Some(v), ..) => write_integer!(buff, *v),
            Value::Float32(Some(v), ..) => write_float!(self, context, buff, *v),
            Value::Float64(Some(v), ..) => write_float!(self, context, buff, *v),
            Value::Decimal(Some(v), ..) => drop(write!(buff, "{}", v)),
            Value::Char(Some(v), ..) => {
                let _ = buff.write_char('\'');
                let _ = buff.write_char(*v);
                let _ = buff.write_char('\'');
            }
            Value::Varchar(Some(v), ..) => self.write_value_string(context, buff, v),
            Value::Blob(Some(v), ..) => self.write_value_blob(context, buff, v.as_ref()),
            Value::Date(Some(v), ..) => {
                let _ = buff.write_char('\'');
                self.write_value_date(context, buff, v);
                let _ = buff.write_char('\'');
            }
            Value::Time(Some(v), ..) => {
                let _ = buff.write_char('\'');
                self.write_value_time(context, buff, v);
                let _ = buff.write_char('\'');
            }
            Value::Timestamp(Some(v), ..) => {
                let _ = buff.write_char('\'');
                self.write_value_date(context, buff, &v.date());
                let _ = buff.write_char('T');
                self.write_value_time(context, buff, &v.time());
                let _ = buff.write_char('\'');
            }
            Value::TimestampWithTimezone(Some(v), ..) => {
                let _ = buff.write_char('\'');
                self.write_value_date(context, buff, &v.date());
                let _ = buff.write_char('T');
                self.write_value_time(context, buff, &v.time());
                let _ = write!(
                    buff,
                    "{:+02}:{:02}",
                    v.offset().whole_hours(),
                    v.offset().whole_minutes()
                );
                let _ = buff.write_char('\'');
            }
            Value::Interval(Some(v), ..) => self.write_value_interval(context, buff, v),
            Value::Uuid(Some(v), ..) => drop(write!(buff, "'{}'", v)),
            Value::List(Some(..), ..) | Value::Array(Some(..), ..) => {
                let v = match value {
                    Value::List(Some(v), ..) => v.iter(),
                    Value::Array(Some(v), ..) => v.iter(),
                    _ => unreachable!(),
                };
                let _ = buff.write_char('[');
                separated_by(
                    buff,
                    v,
                    |buff, v| {
                        self.write_value(context, buff, v);
                        true
                    },
                    ",",
                );
                let _ = buff.write_char(']');
            }
            Value::Map(Some(v), ..) => self.write_value_map(context, buff, v),
            Value::Struct(Some(_v), ..) => {
                todo!()
            }
        };
    }

    fn write_value_none(&self, _context: Context, buff: &mut dyn Write) {
        let _ = buff.write_str("NULL");
    }

    fn write_value_bool(&self, _context: Context, buff: &mut dyn Write, value: bool) {
        let _ = buff.write_str(["false", "true"][value as usize]);
    }

    fn write_value_infinity(&self, context: Context, buff: &mut dyn Write, negative: bool) {
        let mut buffer = ryu::Buffer::new();
        self.write_expression_binary_op(
            context,
            buff,
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

    fn write_value_string(&self, _context: Context, buff: &mut dyn Write, value: &str) {
        let _ = buff.write_char('\'');
        let mut position = 0;
        for (i, c) in value.char_indices() {
            if c == '\'' {
                let _ = buff.write_str(&value[position..i]);
                let _ = buff.write_str("''");
                position = i + 1;
            } else if c == '\n' {
                let _ = buff.write_str(&value[position..i]);
                let _ = buff.write_str("\\n");
                position = i + 1;
            }
        }
        let _ = buff.write_str(&value[position..]);
        let _ = buff.write_char('\'');
    }

    fn write_value_blob(&self, _context: Context, buff: &mut dyn Write, value: &[u8]) {
        let _ = buff.write_char('\'');
        for b in value {
            let _ = write!(buff, "\\x{:X}", b);
        }
        let _ = buff.write_char('\'');
    }

    fn write_value_date(&self, _context: Context, buff: &mut dyn Write, value: &Date) {
        let _ = write!(
            buff,
            "{:04}-{:02}-{:02}",
            value.year(),
            value.month() as u8,
            value.day()
        );
    }

    fn write_value_time(&self, _context: Context, buff: &mut dyn Write, value: &Time) {
        let mut subsecond = value.nanosecond();
        let mut width = 9;
        while width > 1 && subsecond % 10 == 0 {
            subsecond /= 10;
            width -= 1;
        }
        let _ = write!(
            buff,
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

    fn write_value_interval(&self, _context: Context, buff: &mut dyn Write, value: &Interval) {
        let _ = buff.write_str("INTERVAL ");
        macro_rules! write_unit {
            ($buff:ident, $val:expr, $unit:expr) => {
                let _ = write!(
                    $buff,
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
            let _ = buff.write_char('\'');
        }
        if months != 0 {
            if months % 12 == 0 {
                write_unit!(buff, months / 12, "YEAR");
            } else {
                write_unit!(buff, months, "MONTH");
            }
        }
        for &(name, factor) in self.value_interval_units() {
            if nanos % factor == 0 {
                let value = nanos / factor;
                if value != 0 {
                    if months != 0 {
                        let _ = buff.write_char(' ');
                    }
                    write_unit!(buff, value, name);
                    break;
                }
            }
        }
        if multiple_units {
            let _ = buff.write_char('\'');
        }
    }

    fn write_value_map(
        &self,
        context: Context,
        buff: &mut dyn Write,
        value: &HashMap<Value, Value>,
    ) {
        let _ = buff.write_char('{');
        separated_by(
            buff,
            value,
            |buff, (k, v)| {
                self.write_value(context, buff, k);
                let _ = buff.write_char(':');
                self.write_value(context, buff, v);
                true
            },
            ",",
        );
        let _ = buff.write_char('}');
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

    fn write_expression_operand(&self, context: Context, buff: &mut dyn Write, value: &Operand) {
        let _ = match value {
            Operand::LitBool(v) => self.write_value_bool(context, buff, *v),
            Operand::LitFloat(v) => write_float!(self, context, buff, *v),
            Operand::LitIdent(v) => drop(buff.write_str(v)),
            Operand::LitField(v) => {
                separated_by(buff, *v, |buff, v| buff.write_str(v).is_ok(), ".")
            }
            Operand::LitInt(v) => write_integer!(buff, *v),
            Operand::LitStr(v) => self.write_value_string(context, buff, v),
            Operand::LitArray(v) => {
                let _ = buff.write_char('[');
                separated_by(
                    buff,
                    *v,
                    |buff, v| {
                        v.write_query(self.as_dyn(), context, buff);
                        true
                    },
                    ", ",
                );
                let _ = buff.write_char(']');
            }
            Operand::Null => drop(buff.write_str("NULL")),
            Operand::Type(v) => self.write_column_type(context, buff, v),
            Operand::Variable(v) => self.write_value(context, buff, v),
            Operand::Call(f, args) => {
                let _ = buff.write_str(f);
                let _ = buff.write_char('(');
                separated_by(
                    buff,
                    *args,
                    |buff, v| {
                        v.write_query(self.as_dyn(), context, buff);
                        true
                    },
                    ",",
                );
                let _ = buff.write_char(')');
            }
            Operand::Asterisk => drop(buff.write_char('*')),
            Operand::QuestionMark => drop(buff.write_char('?')),
        };
    }

    fn write_expression_unary_op(
        self: &Self,
        context: Context,
        buff: &mut dyn Write,
        value: &UnaryOp<&dyn Expression>,
    ) {
        let _ = match value.op {
            UnaryOpType::Negative => buff.write_char('-'),
            UnaryOpType::Not => buff.write_str("NOT "),
        };
        possibly_parenthesized!(
            buff,
            value.arg.precedence(self.as_dyn()) <= self.expression_unary_op_precedence(&value.op),
            value.arg.write_query(self.as_dyn(), context, buff)
        );
    }

    fn write_expression_binary_op(
        &self,
        context: Context,
        buff: &mut dyn Write,
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
        let _ = buff.write_str(prefix);
        possibly_parenthesized!(
            buff,
            !lhs_parenthesized && value.lhs.precedence(self.as_dyn()) < precedence,
            value.lhs.write_query(self.as_dyn(), context, buff)
        );
        let _ = buff.write_str(infix);
        possibly_parenthesized!(
            buff,
            !rhs_parenthesized && value.rhs.precedence(self.as_dyn()) <= precedence,
            value.rhs.write_query(self.as_dyn(), context, buff)
        );
        let _ = buff.write_str(suffix);
    }

    fn write_expression_ordered(
        &self,
        context: Context,
        buff: &mut dyn Write,
        value: &dyn Expression,
        order: Order,
    ) {
        value.write_query(self.as_dyn(), context, buff);
        if context.fragment == Fragment::SqlSelectOrderBy {
            let _ = write!(
                buff,
                " {}",
                match order {
                    Order::ASC => "ASC",
                    Order::DESC => "DESC",
                }
            );
        }
    }

    fn write_join_type(&self, _context: Context, buff: &mut dyn Write, join_type: &JoinType) {
        let _ = buff.write_str(match &join_type {
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
        buff: &mut dyn Write,
        join: &Join<&dyn DataSet, &dyn DataSet, &dyn Expression>,
    ) {
        let context = Context {
            fragment: Fragment::SqlJoin,
            qualify_columns: true,
        };
        join.lhs.write_query(self.as_dyn(), context, buff);
        let _ = buff.write_char(' ');
        self.write_join_type(context, buff, &join.join);
        let _ = buff.write_char(' ');
        join.rhs.write_query(self.as_dyn(), context, buff);
        if let Some(on) = &join.on {
            let _ = buff.write_str(" ON ");
            let context = Context {
                qualify_columns: true,
                ..context
            };
            on.write_query(self.as_dyn(), context, buff);
        }
    }

    fn write_create_schema<E, Buff>(&self, buff: &mut Buff, if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
        Buff: Write,
    {
        let _ = buff.write_str("CREATE SCHEMA ");
        let context = Context {
            fragment: Fragment::SqlCreateSchema,
            qualify_columns: E::qualified_columns(),
        };
        if if_not_exists {
            let _ = buff.write_str("IF NOT EXISTS ");
        }
        self.write_identifier_quoted(context, buff, E::table_ref().schema);
        let _ = buff.write_char(';');
    }

    fn write_drop_schema<E, Buff>(&self, buff: &mut Buff, if_exists: bool)
    where
        Self: Sized,
        E: Entity,
        Buff: Write,
    {
        let _ = buff.write_str("DROP SCHEMA ");
        let context = Context {
            fragment: Fragment::SqlDropSchema,
            qualify_columns: E::qualified_columns(),
        };
        if if_exists {
            let _ = buff.write_str("IF EXISTS ");
        }
        self.write_identifier_quoted(context, buff, E::table_ref().schema);
        let _ = buff.write_char(';');
    }

    fn write_create_table<E, Buff>(&self, buff: &mut Buff, if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
        Buff: Write,
    {
        let _ = buff.write_str("CREATE TABLE ");
        if if_not_exists {
            let _ = buff.write_str("IF NOT EXISTS ");
        }
        let context = Context {
            fragment: Fragment::SqlCreateTable,
            qualify_columns: E::qualified_columns(),
        };
        self.write_table_ref(context, buff, E::table_ref());
        let _ = buff.write_str(" (\n");
        separated_by(
            buff,
            E::columns(),
            |buff, v| {
                self.write_create_table_column_fragment(context, buff, v);
                true
            },
            ",\n",
        );
        let primary_key = E::primary_key_def();
        if primary_key.len() > 1 {
            let _ = buff.write_str(",\nPRIMARY KEY (");
            separated_by(
                buff,
                primary_key,
                |buff, v| {
                    self.write_identifier_quoted(
                        context.with_context(Fragment::SqlCreateTablePrimaryKey),
                        buff,
                        v.name(),
                    );
                    true
                },
                ", ",
            );
            let _ = buff.write_char(')');
        }
        for unique in E::unique_defs() {
            if unique.len() > 1 {
                let _ = buff.write_str(",\nUNIQUE (");
                separated_by(
                    buff,
                    unique,
                    |buff, v| {
                        self.write_identifier_quoted(
                            context.with_context(Fragment::SqlCreateTableUnique),
                            buff,
                            v.name(),
                        );
                        true
                    },
                    ", ",
                );
                let _ = buff.write_char(')');
            }
        }
        let _ = buff.write_str("\n)");
        let _ = buff.write_char(';');
        self.write_column_comments::<E>(context, buff);
    }

    fn write_column_comments<E>(&self, _context: Context, buff: &mut dyn Write)
    where
        Self: Sized,
        E: Entity,
    {
        let context = Context {
            fragment: Fragment::SqlCommentOnColumn,
            qualify_columns: true,
        };
        for c in E::columns().iter().filter(|c| !c.comment.is_empty()) {
            let _ = buff.write_str("\nCOMMENT ON COLUMN ");
            self.write_column_ref(context, buff, c.into());
            let _ = buff.write_str(" IS ");
            self.write_value_string(context, buff, c.comment);
            let _ = buff.write_char(';');
        }
    }

    fn write_create_table_column_fragment<Buff>(
        &self,
        context: Context,
        buff: &mut Buff,
        column: &ColumnDef,
    ) where
        Self: Sized,
        Buff: Write,
    {
        self.write_identifier_quoted(context, buff, &column.name());
        let _ = buff.write_char(' ');
        if !column.column_type.is_empty() {
            let _ = buff.write_str(&column.column_type);
        } else {
            SqlWriter::write_column_type(self, context, buff, &column.value);
        }
        if !column.nullable && column.primary_key == PrimaryKeyType::None {
            let _ = buff.write_str(" NOT NULL");
        }
        if let Some(default) = &column.default {
            let _ = buff.write_str(" DEFAULT ");
            default.write_query(self.as_dyn(), context, buff);
        }
        if column.primary_key == PrimaryKeyType::PrimaryKey {
            // Composite primary key will be printed elsewhere
            let _ = buff.write_str(" PRIMARY KEY");
        }
        if column.unique && column.primary_key != PrimaryKeyType::PrimaryKey {
            let _ = buff.write_str(" UNIQUE");
        }
        if let Some(references) = column.references {
            let _ = buff.write_str(" REFERENCES ");
            self.write_table_ref(context, buff, &references.table_ref());
            let _ = buff.write_char('(');
            self.write_column_ref(context, buff, &references);
            let _ = buff.write_char(')');
        }
    }

    fn write_drop_table<E, Buff>(&self, buff: &mut Buff, if_exists: bool)
    where
        Self: Sized,
        E: Entity,
        Buff: Write,
    {
        let _ = buff.write_str("DROP TABLE ");
        let context = Context {
            fragment: Fragment::SqlDropTable,
            qualify_columns: E::qualified_columns(),
        };
        if if_exists {
            let _ = buff.write_str("IF EXISTS ");
        }
        self.write_table_ref(context, buff, E::table_ref());
        let _ = buff.write_char(';');
    }

    fn write_select<Buff, Item, Cols, Data, Cond>(
        &self,
        buff: &mut Buff,
        columns: Cols,
        from: &Data,
        condition: &Cond,
        limit: Option<u32>,
    ) where
        Self: Sized,
        Buff: Write,
        Item: Expression,
        Cols: IntoIterator<Item = Item> + Clone,
        Data: DataSet,
        Cond: Expression,
    {
        let _ = buff.write_str("SELECT ");
        let mut has_order_by = false;
        let context = Context {
            fragment: Fragment::SqlSelect,
            qualify_columns: Data::qualified_columns(),
        };
        separated_by(
            buff,
            columns.clone(),
            |buff, col| {
                col.write_query(self, context, buff);
                has_order_by = has_order_by || col.is_ordered();
                true
            },
            ", ",
        );
        let _ = buff.write_str("\nFROM ");
        from.write_query(self, context.with_context(Fragment::SqlSelectFrom), buff);
        let _ = buff.write_str("\nWHERE ");
        condition.write_query(self, context.with_context(Fragment::SqlSelectWhere), buff);
        if has_order_by {
            let _ = buff.write_str("\nORDER BY ");
            for col in columns.into_iter().filter(Expression::is_ordered) {
                col.write_query(self, context.with_context(Fragment::SqlSelectOrderBy), buff);
            }
        }
        if let Some(limit) = limit {
            let _ = write!(buff, "\nLIMIT {}", limit);
        }
        let _ = buff.write_char(';');
    }

    fn write_insert<'b, E, Buff, It>(&self, buff: &mut Buff, entities: It, update: bool)
    where
        Self: Sized,
        E: Entity + 'b,
        Buff: Write,
        It: IntoIterator<Item = &'b E>,
    {
        let mut rows = entities.into_iter().map(Entity::row_filtered).peekable();
        let Some(mut row) = rows.next() else {
            return;
        };
        let _ = buff.write_str("INSERT INTO ");
        let mut context = Context {
            fragment: Fragment::SqlInsertInto,
            qualify_columns: E::qualified_columns(),
        };
        self.write_table_ref(context, buff, E::table_ref());
        let _ = buff.write_str(" (");
        let columns = E::columns().iter();
        let single = rows.peek().is_none();
        if single {
            // Inserting a single row uses row_labeled to filter buff Passive::NotSet columns
            separated_by(
                buff,
                row.iter(),
                |buff, v| {
                    self.write_identifier_quoted(context, buff, v.0);
                    true
                },
                ", ",
            );
        } else {
            // Inserting more rows will list all columns, Passive::NotSet columns will result in DEFAULT value
            separated_by(
                buff,
                columns.clone(),
                |buff, v| {
                    self.write_identifier_quoted(context, buff, v.name());
                    true
                },
                ", ",
            );
        };
        let _ = buff.write_str(") VALUES\n");
        context.fragment = Fragment::SqlInsertIntoValues;
        let mut first_row = None;
        let mut separate = false;
        loop {
            if separate {
                let _ = buff.write_str(",\n");
            }
            let _ = buff.write_char('(');
            let mut fields = row.iter();
            let mut field = fields.next();
            separated_by(
                buff,
                E::columns(),
                |buff, col| {
                    if Some(col.name()) == field.map(|v| v.0) {
                        self.write_value(
                            context,
                            buff,
                            field
                                .map(|v| &v.1)
                                .expect(&format!("Column {} does not have a value", col.name())),
                        );
                        field = fields.next();
                        true
                    } else if !single {
                        let _ = buff.write_str("DEFAULT");
                        true
                    } else {
                        false
                    }
                },
                ", ",
            );
            let _ = buff.write_char(')');
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
            self.write_insert_update_fragment::<E, _, _>(
                context,
                buff,
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
        let _ = buff.write_char(';');
    }

    fn write_insert_update_fragment<'a, E, Buff, It>(
        &self,
        mut context: Context,
        buff: &mut Buff,
        columns: It,
    ) where
        Self: Sized,
        E: Entity,
        Buff: Write,
        It: Iterator<Item = &'a ColumnDef>,
    {
        let pk = E::primary_key_def();
        if pk.len() == 0 {
            return;
        }
        let _ = buff.write_str("\nON CONFLICT");
        context.fragment = Fragment::SqlInsertIntoOnConflict;
        if pk.len() > 0 {
            let _ = buff.write_str(" (");
            separated_by(
                buff,
                pk,
                |buff, v| {
                    self.write_identifier_quoted(context, buff, v.name());
                    true
                },
                ", ",
            );
            let _ = buff.write_char(')');
        }
        let _ = buff.write_str(" DO UPDATE SET\n");
        separated_by(
            buff,
            columns.filter(|c| c.primary_key == PrimaryKeyType::None),
            |buff, v| {
                self.write_identifier_quoted(context, buff, v.name());
                let _ = buff.write_str(" = EXCLUDED.");
                self.write_identifier_quoted(context, buff, v.name());
                true
            },
            ",\n",
        );
    }

    fn write_delete<E: Entity, Buff: Write, Expr: Expression>(
        &self,
        buff: &mut Buff,
        condition: &Expr,
    ) where
        Self: Sized,
    {
        let _ = buff.write_str("DELETE FROM ");
        let context = Context {
            fragment: Fragment::SqlDeleteFrom,
            qualify_columns: E::qualified_columns(),
        };
        self.write_table_ref(context, buff, E::table_ref());
        let _ = buff.write_str("\nWHERE ");
        condition.write_query(
            self,
            context.with_context(Fragment::SqlDeleteFromWhere),
            buff,
        );
        let _ = buff.write_char(';');
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
