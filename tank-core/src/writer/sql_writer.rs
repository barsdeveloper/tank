use crate::{
    Action, BinaryOp, BinaryOpType, ColumnDef, ColumnRef, DataSet, EitherIterator, Entity,
    Expression, Fragment, Interval, Join, JoinType, Operand, Order, Ordered, PrimaryKeyType,
    TableRef, UnaryOp, UnaryOpType, Value, possibly_parenthesized, separated_by, writer::Context,
};
use futures::future::Either;
use std::{collections::HashMap, fmt::Write};
use time::{Date, Time};

macro_rules! write_integer {
    ($buff:ident, $value:expr) => {{
        let mut buffer = itoa::Buffer::new();
        $buff.push_str(buffer.format($value));
    }};
}
macro_rules! write_float {
    ($this:ident, $context:ident,$buff:ident, $value:expr) => {{
        if $value.is_infinite() {
            $this.write_value_infinity($context, $buff, $value.is_sign_negative());
        } else {
            let mut buffer = ryu::Buffer::new();
            $buff.push_str(buffer.format($value));
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
        _context: Context,
        buff: &mut String,
        value: &str,
        search: char,
        replace: &str,
    ) {
        let mut position = 0;
        for (i, c) in value.char_indices() {
            if c == search {
                buff.push_str(&value[position..i]);
                buff.push_str(replace);
                position = i + 1;
            }
        }
        buff.push_str(&value[position..]);
    }

    fn write_identifier_quoted(&self, context: Context, buff: &mut String, value: &str) {
        buff.push('"');
        self.write_escaped(context, buff, value, '"', r#""""#);
        buff.push('"');
    }

    fn write_table_ref(&self, context: Context, buff: &mut String, value: &TableRef) {
        if self.alias_declaration(context) || value.alias.is_empty() {
            if !value.schema.is_empty() {
                self.write_identifier_quoted(context, buff, &value.schema);
                buff.push('.');
            }
            self.write_identifier_quoted(context, buff, &value.name);
        }
        if !value.alias.is_empty() {
            let _ = write!(buff, " {}", value.alias);
        }
    }

    fn write_column_ref(&self, context: Context, buff: &mut String, value: &ColumnRef) {
        if context.qualify_columns && !value.table.is_empty() {
            if !value.schema.is_empty() {
                self.write_identifier_quoted(context, buff, &value.schema);
                buff.push('.');
            }
            self.write_identifier_quoted(context, buff, &value.table);
            buff.push('.');
        }
        self.write_identifier_quoted(context, buff, &value.name);
    }

    fn write_column_type(&self, context: Context, buff: &mut String, value: &Value) {
        match value {
            Value::Boolean(..) => buff.push_str("BOOLEAN"),
            Value::Int8(..) => buff.push_str("TINYINT"),
            Value::Int16(..) => buff.push_str("SMALLINT"),
            Value::Int32(..) => buff.push_str("INTEGER"),
            Value::Int64(..) => buff.push_str("BIGINT"),
            Value::Int128(..) => buff.push_str("HUGEINT"),
            Value::UInt8(..) => buff.push_str("UTINYINT"),
            Value::UInt16(..) => buff.push_str("USMALLINT"),
            Value::UInt32(..) => buff.push_str("UINTEGER"),
            Value::UInt64(..) => buff.push_str("UBIGINT"),
            Value::UInt128(..) => buff.push_str("UHUGEINT"),
            Value::Float32(..) => buff.push_str("FLOAT"),
            Value::Float64(..) => buff.push_str("DOUBLE"),
            Value::Decimal(.., precision, scale) => {
                buff.push_str("DECIMAL");
                if (precision, scale) != (&0, &0) {
                    let _ = write!(buff, "({},{})", precision, scale);
                }
            }
            Value::Char(..) => buff.push_str("CHAR(1)"),
            Value::Varchar(..) => buff.push_str("VARCHAR"),
            Value::Blob(..) => buff.push_str("BLOB"),
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
            Value::Map(.., key, value) => {
                buff.push_str("MAP(");
                self.write_column_type(context, buff, key);
                buff.push(',');
                self.write_column_type(context, buff, value);
                buff.push(')');
            }
            _ => log::error!(
                "Unexpected tank::Value, variant {:?} is not supported",
                value
            ),
        };
    }

    fn write_value(&self, context: Context, buff: &mut String, value: &Value) {
        match value {
            v if v.is_null() => self.write_value_none(context, buff),
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
                buff.push('\'');
                buff.push(*v);
                buff.push('\'');
            }
            Value::Varchar(Some(v), ..) => self.write_value_string(context, buff, v),
            Value::Blob(Some(v), ..) => self.write_value_blob(context, buff, v.as_ref()),
            Value::Date(Some(v), ..) => {
                buff.push('\'');
                self.write_value_date(context, buff, v);
                buff.push('\'');
            }
            Value::Time(Some(v), ..) => {
                buff.push('\'');
                self.write_value_time(context, buff, v);
                buff.push('\'');
            }
            Value::Timestamp(Some(v), ..) => {
                buff.push('\'');
                self.write_value_date(context, buff, &v.date());
                buff.push('T');
                self.write_value_time(context, buff, &v.time());
                buff.push('\'');
            }
            Value::TimestampWithTimezone(Some(v), ..) => {
                buff.push('\'');
                self.write_value_date(context, buff, &v.date());
                buff.push('T');
                self.write_value_time(context, buff, &v.time());
                let _ = write!(
                    buff,
                    "{:+02}:{:02}",
                    v.offset().whole_hours(),
                    v.offset().whole_minutes()
                );
                buff.push('\'');
            }
            Value::Interval(Some(v), ..) => self.write_value_interval(context, buff, v),
            Value::Uuid(Some(v), ..) => drop(write!(buff, "'{}'", v)),
            Value::Array(Some(..), ..) | Value::List(Some(..), ..) => {
                self.write_value_list(
                    context,
                    buff,
                    match value {
                        Value::Array(Some(v), ..) => Either::Left(v),
                        Value::List(Some(v), ..) => Either::Right(v),
                        _ => unreachable!(),
                    },
                );
            }
            Value::Map(Some(v), ..) => self.write_value_map(context, buff, v),
            Value::Struct(Some(_v), ..) => {
                todo!()
            }
            _ => {
                log::error!("Cannot write {:?}", value);
            }
        };
    }

    fn write_value_none(&self, _context: Context, buff: &mut String) {
        buff.push_str("NULL");
    }

    fn write_value_bool(&self, _context: Context, buff: &mut String, value: bool) {
        buff.push_str(["false", "true"][value as usize]);
    }

    fn write_value_infinity(&self, context: Context, buff: &mut String, negative: bool) {
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

    fn write_value_string(&self, _context: Context, buff: &mut String, value: &str) {
        buff.push('\'');
        let mut position = 0;
        for (i, c) in value.char_indices() {
            if c == '\'' {
                buff.push_str(&value[position..i]);
                buff.push_str("''");
                position = i + 1;
            } else if c == '\n' {
                buff.push_str(&value[position..i]);
                buff.push_str("\\n");
                position = i + 1;
            }
        }
        buff.push_str(&value[position..]);
        buff.push('\'');
    }

    fn write_value_blob(&self, _context: Context, buff: &mut String, value: &[u8]) {
        buff.push('\'');
        for b in value {
            let _ = write!(buff, "\\x{:X}", b);
        }
        buff.push('\'');
    }

    fn write_value_date(&self, _context: Context, buff: &mut String, value: &Date) {
        let _ = write!(
            buff,
            "{:04}-{:02}-{:02}",
            value.year(),
            value.month() as u8,
            value.day()
        );
    }

    fn write_value_time(&self, _context: Context, buff: &mut String, value: &Time) {
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

    fn write_value_interval(&self, _context: Context, buff: &mut String, value: &Interval) {
        buff.push_str("INTERVAL ");
        if value.is_zero() {
            buff.push_str("0 SECONDS");
            return;
        }
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
            buff.push('\'');
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
                        buff.push(' ');
                    }
                    write_unit!(buff, value, name);
                    break;
                }
            }
        }
        if multiple_units {
            buff.push('\'');
        }
    }

    fn write_value_list<'a>(
        &self,
        context: Context,
        buff: &mut String,
        value: Either<&Box<[Value]>, &Vec<Value>>,
    ) {
        buff.push('[');
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
        buff.push(']');
    }

    fn write_value_map(&self, context: Context, buff: &mut String, value: &HashMap<Value, Value>) {
        buff.push('{');
        separated_by(
            buff,
            value,
            |buff, (k, v)| {
                self.write_value(context, buff, k);
                buff.push(':');
                self.write_value(context, buff, v);
            },
            ",",
        );
        buff.push('}');
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

    fn write_expression_operand(&self, context: Context, buff: &mut String, value: &Operand) {
        match value {
            Operand::LitBool(v) => self.write_value_bool(context, buff, *v),
            Operand::LitFloat(v) => write_float!(self, context, buff, *v),
            Operand::LitIdent(v) => drop(buff.push_str(v)),
            Operand::LitField(v) => separated_by(buff, *v, |buff, v| buff.push_str(v), "."),
            Operand::LitInt(v) => write_integer!(buff, *v),
            Operand::LitStr(v) => self.write_value_string(context, buff, v),
            Operand::LitArray(v) => {
                buff.push('[');
                separated_by(
                    buff,
                    *v,
                    |buff, v| {
                        v.write_query(self.as_dyn(), context, buff);
                    },
                    ", ",
                );
                buff.push(']');
            }
            Operand::Null => drop(buff.push_str("NULL")),
            Operand::Type(v) => self.write_column_type(context, buff, v),
            Operand::Variable(v) => self.write_value(context, buff, v),
            Operand::Call(f, args) => {
                buff.push_str(f);
                buff.push('(');
                separated_by(
                    buff,
                    *args,
                    |buff, v| {
                        v.write_query(self.as_dyn(), context, buff);
                    },
                    ",",
                );
                buff.push(')');
            }
            Operand::Asterisk => drop(buff.push('*')),
            Operand::QuestionMark => drop(buff.push('?')),
        };
    }

    fn write_expression_unary_op(
        self: &Self,
        context: Context,
        buff: &mut String,
        value: &UnaryOp<&dyn Expression>,
    ) {
        match value.op {
            UnaryOpType::Negative => buff.push('-'),
            UnaryOpType::Not => buff.push_str("NOT "),
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
        buff: &mut String,
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
        buff.push_str(prefix);
        possibly_parenthesized!(
            buff,
            !lhs_parenthesized && value.lhs.precedence(self.as_dyn()) < precedence,
            value.lhs.write_query(self.as_dyn(), context, buff)
        );
        buff.push_str(infix);
        possibly_parenthesized!(
            buff,
            !rhs_parenthesized && value.rhs.precedence(self.as_dyn()) <= precedence,
            value.rhs.write_query(self.as_dyn(), context, buff)
        );
        buff.push_str(suffix);
    }

    fn write_expression_ordered(
        &self,
        context: Context,
        buff: &mut String,
        value: &Ordered<&dyn Expression>,
    ) {
        value.expression.write_query(self.as_dyn(), context, buff);
        if context.fragment == Fragment::SqlSelectOrderBy {
            let _ = write!(
                buff,
                " {}",
                match value.order {
                    Order::ASC => "ASC",
                    Order::DESC => "DESC",
                }
            );
        }
    }

    fn write_join_type(&self, _context: Context, buff: &mut String, join_type: &JoinType) {
        buff.push_str(match &join_type {
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
        buff: &mut String,
        join: &Join<&dyn DataSet, &dyn DataSet, &dyn Expression>,
    ) {
        let context = Context {
            fragment: Fragment::SqlJoin,
            qualify_columns: true,
        };
        join.lhs.write_query(self.as_dyn(), context, buff);
        buff.push(' ');
        self.write_join_type(context, buff, &join.join);
        buff.push(' ');
        join.rhs.write_query(self.as_dyn(), context, buff);
        if let Some(on) = &join.on {
            buff.push_str(" ON ");
            let context = Context {
                qualify_columns: true,
                ..context
            };
            on.write_query(self.as_dyn(), context, buff);
        }
    }

    fn write_transaction_begin(&self, buff: &mut String) {
        buff.push_str("BEGIN;");
    }

    fn write_transaction_commit(&self, buff: &mut String) {
        buff.push_str("COMMIT;");
    }

    fn write_transaction_rollback(&self, buff: &mut String) {
        buff.push_str("ROLLBACK;");
    }

    fn write_create_schema<E>(&self, buff: &mut String, if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        buff.push_str("CREATE SCHEMA ");
        let context = Context {
            fragment: Fragment::SqlCreateSchema,
            qualify_columns: E::qualified_columns(),
        };
        if if_not_exists {
            buff.push_str("IF NOT EXISTS ");
        }
        self.write_identifier_quoted(context, buff, E::table_ref().schema);
        buff.push(';');
    }

    fn write_drop_schema<E>(&self, buff: &mut String, if_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        buff.push_str("DROP SCHEMA ");
        let context = Context {
            fragment: Fragment::SqlDropSchema,
            qualify_columns: E::qualified_columns(),
        };
        if if_exists {
            buff.push_str("IF EXISTS ");
        }
        self.write_identifier_quoted(context, buff, E::table_ref().schema);
        buff.push(';');
    }

    fn write_create_table<E>(&self, buff: &mut String, if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        buff.push_str("CREATE TABLE ");
        if if_not_exists {
            buff.push_str("IF NOT EXISTS ");
        }
        let context = Context {
            fragment: Fragment::SqlCreateTable,
            qualify_columns: E::qualified_columns(),
        };
        self.write_table_ref(context, buff, E::table_ref());
        buff.push_str(" (\n");
        separated_by(
            buff,
            E::columns(),
            |buff, v| {
                self.write_create_table_column_fragment(context, buff, v);
            },
            ",\n",
        );
        let primary_key = E::primary_key_def();
        if primary_key.len() > 1 {
            buff.push_str(",\nPRIMARY KEY (");
            separated_by(
                buff,
                primary_key,
                |buff, v| {
                    self.write_identifier_quoted(
                        context.with_context(Fragment::SqlCreateTablePrimaryKey),
                        buff,
                        v.name(),
                    );
                },
                ", ",
            );
            buff.push(')');
        }
        for unique in E::unique_defs() {
            if unique.len() > 1 {
                buff.push_str(",\nUNIQUE (");
                separated_by(
                    buff,
                    unique,
                    |buff, v| {
                        self.write_identifier_quoted(
                            context.with_context(Fragment::SqlCreateTableUnique),
                            buff,
                            v.name(),
                        );
                    },
                    ", ",
                );
                buff.push(')');
            }
        }
        buff.push_str(");");
        self.write_column_comments::<E>(context, buff);
    }

    fn write_column_comments<E>(&self, _context: Context, buff: &mut String)
    where
        Self: Sized,
        E: Entity,
    {
        let context = Context {
            fragment: Fragment::SqlCommentOnColumn,
            qualify_columns: true,
        };
        for c in E::columns().iter().filter(|c| !c.comment.is_empty()) {
            buff.push_str("\nCOMMENT ON COLUMN ");
            self.write_column_ref(context, buff, c.into());
            buff.push_str(" IS ");
            self.write_value_string(context, buff, c.comment);
            buff.push(';');
        }
    }

    fn write_create_table_column_fragment(
        &self,
        context: Context,
        buff: &mut String,
        column: &ColumnDef,
    ) where
        Self: Sized,
    {
        self.write_identifier_quoted(context, buff, &column.name());
        buff.push(' ');
        if !column.column_type.is_empty() {
            buff.push_str(&column.column_type);
        } else {
            SqlWriter::write_column_type(self, context, buff, &column.value);
        }
        if !column.nullable && column.primary_key == PrimaryKeyType::None {
            buff.push_str(" NOT NULL");
        }
        if let Some(default) = &column.default {
            buff.push_str(" DEFAULT ");
            default.write_query(self.as_dyn(), context, buff);
        }
        if column.primary_key == PrimaryKeyType::PrimaryKey {
            // Composite primary key will be printed elsewhere
            buff.push_str(" PRIMARY KEY");
        }
        if column.unique && column.primary_key != PrimaryKeyType::PrimaryKey {
            buff.push_str(" UNIQUE");
        }
        if let Some(references) = column.references {
            buff.push_str(" REFERENCES ");
            self.write_table_ref(context, buff, &references.table_ref());
            buff.push('(');
            self.write_column_ref(context, buff, &references);
            buff.push(')');
            if let Some(on_delete) = &column.on_delete {
                buff.push_str(" ON DELETE ");
                self.write_create_table_references_action(context, buff, on_delete);
            }
            if let Some(on_update) = &column.on_update {
                buff.push_str(" ON UPDATE ");
                self.write_create_table_references_action(context, buff, on_update);
            }
        }
    }

    fn write_create_table_references_action(
        &self,
        _context: Context,
        buff: &mut String,
        action: &Action,
    ) {
        buff.push_str(match action {
            Action::NoAction => "NO ACTION",
            Action::Restrict => "RESTRICT",
            Action::Cascade => "CASCADE",
            Action::SetNull => "SET NULL",
            Action::SetDefault => "SET DEFAULT",
        });
    }

    fn write_drop_table<E>(&self, buff: &mut String, if_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        buff.push_str("DROP TABLE ");
        let context = Context {
            fragment: Fragment::SqlDropTable,
            qualify_columns: E::qualified_columns(),
        };
        if if_exists {
            buff.push_str("IF EXISTS ");
        }
        self.write_table_ref(context, buff, E::table_ref());
        buff.push(';');
    }

    fn write_select<Item, Cols, Data, Cond>(
        &self,
        buff: &mut String,
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
        buff.push_str("SELECT ");
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
            },
            ", ",
        );
        buff.push_str("\nFROM ");
        from.write_query(self, context.with_context(Fragment::SqlSelectFrom), buff);
        buff.push_str("\nWHERE ");
        condition.write_query(self, context.with_context(Fragment::SqlSelectWhere), buff);
        if has_order_by {
            buff.push_str("\nORDER BY ");
            for col in columns.into_iter().filter(Expression::is_ordered) {
                col.write_query(self, context.with_context(Fragment::SqlSelectOrderBy), buff);
            }
        }
        if let Some(limit) = limit {
            let _ = write!(buff, "\nLIMIT {}", limit);
        }
        buff.push(';');
    }

    fn write_insert<'b, E, It>(&self, buff: &mut String, entities: It, update: bool)
    where
        Self: Sized,
        E: Entity + 'b,
        It: IntoIterator<Item = &'b E>,
    {
        let mut rows = entities.into_iter().map(Entity::row_filtered).peekable();
        let Some(mut row) = rows.next() else {
            return;
        };
        buff.push_str("INSERT INTO ");
        let mut context = Context {
            fragment: Fragment::SqlInsertInto,
            qualify_columns: E::qualified_columns(),
        };
        self.write_table_ref(context, buff, E::table_ref());
        buff.push_str(" (");
        let columns = E::columns().iter();
        let single = rows.peek().is_none();
        if single {
            // Inserting a single row uses row_labeled to filter buff Passive::NotSet columns
            separated_by(
                buff,
                row.iter(),
                |buff, v| {
                    self.write_identifier_quoted(context, buff, v.0);
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
                },
                ", ",
            );
        };
        buff.push_str(") VALUES\n");
        context.fragment = Fragment::SqlInsertIntoValues;
        let mut first_row = None;
        let mut separate = false;
        loop {
            if separate {
                buff.push_str(",\n");
            }
            buff.push('(');
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
                    } else if !single {
                        buff.push_str("DEFAULT");
                    }
                },
                ", ",
            );
            buff.push(')');
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
        buff.push(';');
    }

    fn write_insert_update_fragment<'a, E, It>(
        &self,
        mut context: Context,
        buff: &mut String,
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
        buff.push_str("\nON CONFLICT");
        context.fragment = Fragment::SqlInsertIntoOnConflict;
        if pk.len() > 0 {
            buff.push_str(" (");
            separated_by(
                buff,
                pk,
                |buff, v| {
                    self.write_identifier_quoted(context, buff, v.name());
                },
                ", ",
            );
            buff.push(')');
        }
        buff.push_str(" DO UPDATE SET\n");
        separated_by(
            buff,
            columns.filter(|c| c.primary_key == PrimaryKeyType::None),
            |buff, v| {
                self.write_identifier_quoted(context, buff, v.name());
                buff.push_str(" = EXCLUDED.");
                self.write_identifier_quoted(context, buff, v.name());
            },
            ",\n",
        );
    }

    fn write_delete<E, Expr>(&self, buff: &mut String, condition: &Expr)
    where
        Self: Sized,
        E: Entity,
        Expr: Expression,
    {
        buff.push_str("DELETE FROM ");
        let context = Context {
            fragment: Fragment::SqlDeleteFrom,
            qualify_columns: E::qualified_columns(),
        };
        self.write_table_ref(context, buff, E::table_ref());
        buff.push_str("\nWHERE ");
        condition.write_query(
            self,
            context.with_context(Fragment::SqlDeleteFromWhere),
            buff,
        );
        buff.push(';');
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
