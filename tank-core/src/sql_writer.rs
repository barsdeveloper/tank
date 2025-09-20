use crate::{
    BinaryOp, BinaryOpType, ColumnDef, ColumnRef, DataSet, EitherIterator, Entity, Expression,
    Interval, Join, JoinType, Operand, PrimaryKeyType, TableRef, UnaryOp, UnaryOpType, Value,
    possibly_parenthesized, separated_by,
};
use std::{collections::HashMap, fmt::Write};
use time::{Date, Time};

macro_rules! write_integer {
    ($out:ident, $value:expr) => {{
        let mut buffer = itoa::Buffer::new();
        $out.push_str(buffer.format($value));
    }};
}
macro_rules! write_float {
    ($this:ident, $out:ident, $value:expr) => {{
        let mut buffer = ryu::Buffer::new();
        if $value.is_infinite() {
            $this.write_expression_binary_op(
                $out,
                &BinaryOp {
                    op: BinaryOpType::Cast,
                    lhs: &Operand::LitStr(buffer.format($value)),
                    rhs: &Operand::Type(Value::Float64(None)),
                },
                false,
            );
        } else {
            $out.push_str(buffer.format($value));
        }
    }};
}

pub trait SqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter;

    fn write_escaped(&self, out: &mut String, value: &str, search: char, replace: &str) {
        let mut position = 0;
        for (i, c) in value.char_indices() {
            if c == search {
                out.push_str(&value[position..i]);
                out.push_str(replace);
                position = i + 1;
            }
        }
        out.push_str(&value[position..]);
    }

    fn write_identifier_quoted(&self, out: &mut String, value: &str) {
        out.push('"');
        self.write_escaped(out, value, '"', r#""""#);
        out.push('"');
    }

    fn write_table_ref(&self, out: &mut String, value: &TableRef, is_declaration: bool) {
        if !is_declaration && !value.alias.is_empty() {
            out.push_str(&value.alias);
        } else {
            if !value.schema.is_empty() {
                self.write_identifier_quoted(out, &value.schema);
                out.push('.');
            }
            self.write_identifier_quoted(out, &value.name);
        }
        if is_declaration {
            out.push(' ');
            out.push_str(&value.alias);
        }
    }

    fn write_column_ref(&self, out: &mut String, value: &ColumnRef, qualify: bool) {
        if qualify && !value.table.is_empty() {
            if !value.schema.is_empty() {
                self.write_identifier_quoted(out, &value.schema);
                out.push('.');
            }
            self.write_identifier_quoted(out, &value.table);
            out.push('.');
        }
        self.write_identifier_quoted(out, &value.name);
    }

    fn write_column_type(&self, out: &mut String, value: &Value) {
        match value {
            Value::Boolean(..) => out.push_str("BOOLEAN"),
            Value::Int8(..) => out.push_str("TINYINT"),
            Value::Int16(..) => out.push_str("SMALLINT"),
            Value::Int32(..) => out.push_str("INTEGER"),
            Value::Int64(..) => out.push_str("BIGINT"),
            Value::Int128(..) => out.push_str("HUGEINT"),
            Value::UInt8(..) => out.push_str("UTINYINT"),
            Value::UInt16(..) => out.push_str("USMALLINT"),
            Value::UInt32(..) => out.push_str("UINTEGER"),
            Value::UInt64(..) => out.push_str("UBIGINT"),
            Value::UInt128(..) => out.push_str("UHUGEINT"),
            Value::Float32(..) => out.push_str("FLOAT"),
            Value::Float64(..) => out.push_str("DOUBLE"),
            Value::Decimal(.., precision, scale) => {
                out.push_str("DECIMAL");
                if (precision, scale) != (&0, &0) {
                    let _ = write!(out, "({},{})", precision, scale);
                }
            }
            Value::Char(..) => out.push_str("CHAR(1)"),
            Value::Varchar(..) => out.push_str("VARCHAR"),
            Value::Blob(..) => out.push_str("BLOB"),
            Value::Date(..) => out.push_str("DATE"),
            Value::Time(..) => out.push_str("TIME"),
            Value::Timestamp(..) => out.push_str("TIMESTAMP"),
            Value::TimestampWithTimezone(..) => out.push_str("TIMESTAMP WITH TIME ZONE"),
            Value::Interval(..) => out.push_str("INTERVAL"),
            Value::Uuid(..) => out.push_str("UUID"),
            Value::Array(.., inner, size) => {
                self.write_column_type(out, inner);
                let _ = write!(out, "[{}]", size);
            }
            Value::List(.., inner) => {
                self.write_column_type(out, inner);
                out.push_str("[]");
            }
            Value::Map(.., key, value) => {
                out.push_str("MAP(");
                self.write_column_type(out, key);
                out.push(',');
                self.write_column_type(out, value);
                out.push(')');
            }
            _ => panic!(
                "Unexpected tank::Value, cannot get the sql type from {:?} variant",
                value
            ),
        };
    }

    fn write_value(&self, out: &mut String, value: &Value) {
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
            | Value::Struct(None, ..) => self.write_value_none(out),
            Value::Boolean(Some(v), ..) => self.write_value_bool(out, *v),
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
            Value::Float32(Some(v), ..) => write_float!(self, out, *v),
            Value::Float64(Some(v), ..) => write_float!(self, out, *v),
            Value::Decimal(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Char(Some(v), ..) => {
                out.push('\'');
                out.push(*v);
                out.push('\'');
            }
            Value::Varchar(Some(v), ..) => self.write_value_string(out, v),
            Value::Blob(Some(v), ..) => self.write_value_blob(out, v.as_ref()),
            Value::Date(Some(v), ..) => {
                out.push('\'');
                self.write_value_date(out, v);
                out.push('\'');
            }
            Value::Time(Some(v), ..) => {
                out.push('\'');
                self.write_value_time(out, v);
                out.push('\'');
            }
            Value::Timestamp(Some(v), ..) => {
                out.push('\'');
                self.write_value_date(out, &v.date());
                out.push('T');
                self.write_value_time(out, &v.time());
                out.push('\'');
            }
            Value::TimestampWithTimezone(Some(v), ..) => {
                out.push('\'');
                self.write_value_date(out, &v.date());
                out.push('T');
                self.write_value_time(out, &v.time());
                let _ = write!(
                    out,
                    "{:+02}:{:02}",
                    v.offset().whole_hours(),
                    v.offset().whole_minutes()
                );
                out.push('\'');
            }
            Value::Interval(Some(v), ..) => self.write_value_interval(out, v),
            Value::Uuid(Some(v), ..) => drop(write!(out, "'{}'", v)),
            Value::List(Some(..), ..) | Value::Array(Some(..), ..) => {
                let v = match value {
                    Value::List(Some(v), ..) => v.iter(),
                    Value::Array(Some(v), ..) => v.iter(),
                    _ => unreachable!(),
                };
                out.push('[');
                separated_by(
                    out,
                    v,
                    |out, v| {
                        self.write_value(out, v);
                    },
                    ",",
                );
                out.push(']');
            }
            Value::Map(Some(v), ..) => self.write_value_map(out, v),
            Value::Struct(Some(_v), ..) => {
                todo!()
            }
        };
    }

    fn write_value_none(&self, out: &mut String) {
        out.push_str("NULL")
    }

    fn write_value_bool(&self, out: &mut String, value: bool) {
        out.push_str(["false", "true"][value as usize])
    }

    fn write_value_string(&self, out: &mut String, value: &str) {
        out.push('\'');
        let mut position = 0;
        for (i, c) in value.char_indices() {
            if c == '\'' {
                out.push_str(&value[position..i]);
                out.push_str("''");
                position = i + 1;
            } else if c == '\n' {
                out.push_str(&value[position..i]);
                out.push_str("\\n");
                position = i + 1;
            }
        }
        out.push_str(&value[position..]);
        out.push('\'');
    }

    fn write_value_blob(&self, out: &mut String, value: &[u8]) {
        out.push('\'');
        for b in value {
            let _ = write!(out, "\\x{:X}", b);
        }
        out.push('\'');
    }

    fn write_value_date(&self, out: &mut String, value: &Date) {
        let _ = write!(
            out,
            "{:04}-{:02}-{:02}",
            value.year(),
            value.month() as u8,
            value.day()
        );
    }

    fn write_value_time(&self, out: &mut String, value: &Time) {
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

    fn write_value_interval(&self, out: &mut String, value: &Interval) {
        let _ = out.write_str("INTERVAL");
        let quote_position = out.len() + 1;
        macro_rules! write_unit {
            ($out:ident, $val:expr, $unit:expr) => {
                let _ = write!(
                    $out,
                    " {} {}{}",
                    $val,
                    $unit,
                    if $val > 1 { "S" } else { "" }
                );
            };
        }
        let mut units = 0;
        if value.months != 0 {
            if value.months % 12 == 0 {
                write_unit!(out, value.months / 12, "YEAR");
                units += 1;
            } else {
                write_unit!(out, value.months, "MONTH");
                units += 1;
            }
        }
        let nanos = value.nanos + value.days as i128 * Interval::NANOS_IN_DAY;
        for &(name, factor) in self.value_interval_units() {
            if nanos % factor == 0 {
                let value = nanos / factor;
                if units == 0 || value != 0 {
                    write_unit!(out, value, name);
                    units += 1;
                    break;
                }
            }
        }
        if units > 1 {
            out.insert(quote_position, '\'');
            out.push('\'');
        }
    }

    fn write_value_map(&self, out: &mut String, value: &HashMap<Value, Value>) {
        out.push('{');
        separated_by(
            out,
            value,
            |out, (k, v)| {
                self.write_value(out, k);
                out.push(':');
                self.write_value(out, v);
            },
            ",",
        );
        out.push('}');
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

    fn write_expression_operand(&self, out: &mut String, value: &Operand, qualify_columns: bool) {
        let _ = match value {
            Operand::LitBool(v) => self.write_value_bool(out, *v),
            Operand::LitFloat(v) => write_float!(self, out, *v),
            Operand::LitIdent(v) => out.push_str(v),
            Operand::LitField(v) => separated_by(out, *v, |out, v| out.push_str(v), "."),
            Operand::LitInt(v) => write_integer!(out, *v),
            Operand::LitStr(v) => self.write_value_string(out, v),
            Operand::LitArray(v) => {
                out.push('[');
                separated_by(
                    out,
                    *v,
                    |out, v| {
                        v.write_query(self.as_dyn(), out, qualify_columns);
                    },
                    ", ",
                );
                out.push(']');
            }
            Operand::Null => out.push_str("NULL"),
            Operand::Type(v) => self.write_column_type(out, v),
            Operand::Variable(v) => self.write_value(out, v),
            Operand::Call(f, args) => {
                out.push_str(f);
                out.push('(');
                separated_by(
                    out,
                    *args,
                    |out, v| {
                        v.write_query(self.as_dyn(), out, qualify_columns);
                    },
                    ",",
                );
                out.push(')');
            }
            Operand::Asterisk => out.push('*'),
            Operand::QuestionMark => out.push('?'),
        };
    }

    fn write_expression_unary_op(
        self: &Self,
        out: &mut String,
        value: &UnaryOp<&dyn Expression>,
        qualify_columns: bool,
    ) {
        let _ = match value.op {
            UnaryOpType::Negative => out.push('-'),
            UnaryOpType::Not => out.push_str("NOT "),
        };
        possibly_parenthesized!(
            out,
            value.v.precedence(self.as_dyn()) <= self.expression_unary_op_precedence(&value.op),
            value.v.write_query(self.as_dyn(), out, qualify_columns)
        );
    }

    fn write_expression_binary_op(
        &self,
        out: &mut String,
        value: &BinaryOp<&dyn Expression, &dyn Expression>,
        qualify_columns: bool,
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
        out.push_str(prefix);
        possibly_parenthesized!(
            out,
            !lhs_parenthesized && value.lhs.precedence(self.as_dyn()) < precedence,
            value.lhs.write_query(self.as_dyn(), out, qualify_columns)
        );
        out.push_str(infix);
        possibly_parenthesized!(
            out,
            !rhs_parenthesized && value.rhs.precedence(self.as_dyn()) <= precedence,
            value.rhs.write_query(self.as_dyn(), out, qualify_columns)
        );
        out.push_str(suffix);
    }

    fn write_join_type(&self, out: &mut String, join_type: &JoinType) {
        out.push_str(match &join_type {
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
        out: &mut String,
        join: &Join<&dyn DataSet, &dyn DataSet, &dyn Expression>,
    ) {
        join.lhs.write_query(self.as_dyn(), out);
        out.push(' ');
        self.write_join_type(out, &join.join);
        out.push(' ');
        join.rhs.write_query(self.as_dyn(), out);
        if let Some(on) = &join.on {
            out.push_str(" ON ");
            on.write_query(self.as_dyn(), out, true);
        }
    }

    fn write_create_schema<E>(&self, out: &mut String, if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        out.push_str("CREATE SCHEMA ");
        if if_not_exists {
            out.push_str("IF NOT EXISTS ");
        }
        self.write_identifier_quoted(out, E::table_ref().schema);
        out.push(';');
    }

    fn write_drop_schema<E>(&self, out: &mut String, if_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        out.push_str("DROP SCHEMA ");
        if if_exists {
            out.push_str("IF EXISTS ");
        }
        self.write_identifier_quoted(out, E::table_ref().schema);
        out.push(';');
    }

    fn write_create_table<E>(&self, out: &mut String, if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        out.push_str("CREATE TABLE ");
        if if_not_exists {
            out.push_str("IF NOT EXISTS ");
        }
        self.write_table_ref(out, E::table_ref(), false);
        out.push_str(" (\n");
        separated_by(
            out,
            E::columns(),
            |out, v| {
                self.write_create_table_column_fragment(out, v);
            },
            ",\n",
        );
        let primary_key = E::primary_key_def();
        if primary_key.len() > 1 {
            out.push_str(",\nPRIMARY KEY (");
            separated_by(
                out,
                primary_key,
                |out, v| self.write_identifier_quoted(out, v.name()),
                ", ",
            );
            out.push(')');
        }
        for unique in E::unique_defs() {
            if unique.len() > 1 {
                out.push_str(",\nUNIQUE (");
                separated_by(
                    out,
                    unique,
                    |out, v| self.write_identifier_quoted(out, v.name()),
                    ", ",
                );
                out.push(')');
            }
        }
        out.push_str("\n)");
        out.push(';');
        self.write_column_comments::<E>(out);
    }

    fn write_column_comments<E>(&self, out: &mut String)
    where
        Self: Sized,
        E: Entity,
    {
        for c in E::columns().iter().filter(|c| !c.comment.is_empty()) {
            out.push_str("\nCOMMENT ON COLUMN ");
            self.write_column_ref(out, c.into(), true);
            out.push_str(" IS ");
            self.write_value_string(out, c.comment);
            out.push(';');
        }
    }

    fn write_create_table_column_fragment(&self, out: &mut String, column: &ColumnDef) {
        self.write_identifier_quoted(out, &column.name());
        out.push(' ');
        if !column.column_type.is_empty() {
            out.push_str(&column.column_type);
        } else {
            SqlWriter::write_column_type(self, out, &column.value);
        }
        if !column.nullable && column.primary_key == PrimaryKeyType::None {
            out.push_str(" NOT NULL");
        }
        if let Some(default) = &column.default {
            out.push_str(" DEFAULT ");
            default.write_query(self.as_dyn(), out, true);
        }
        if column.primary_key == PrimaryKeyType::PrimaryKey {
            // Composite primary key will be printed elsewhere
            out.push_str(" PRIMARY KEY");
        }
        if column.unique && column.primary_key != PrimaryKeyType::PrimaryKey {
            out.push_str(" UNIQUE");
        }
        if let Some(references) = column.references {
            out.push_str(" REFERENCES ");
            self.write_table_ref(out, &references.table_ref(), false);
            out.push('(');
            self.write_column_ref(out, &references, false);
            out.push(')');
        }
    }

    fn write_drop_table<E: Entity>(&self, out: &mut String, if_exists: bool)
    where
        Self: Sized,
    {
        out.push_str("DROP TABLE ");
        if if_exists {
            out.push_str("IF EXISTS ");
        }
        self.write_table_ref(out, E::table_ref(), false);
        out.push(';');
    }

    fn write_select<Item, Cols, Data, Cond>(
        &self,
        out: &mut String,
        columns: Cols,
        from: &Data,
        condition: &Cond,
        limit: Option<u32>,
    ) where
        Self: Sized,
        Item: Expression,
        Cols: IntoIterator<Item = Item>,
        Data: DataSet,
        Cond: Expression,
    {
        out.push_str("SELECT ");
        separated_by(
            out,
            columns,
            |out, col| {
                col.write_query(self, out, Data::qualified_columns());
            },
            ", ",
        );
        out.push_str("\nFROM ");
        from.write_query(self, out);
        out.push_str("\nWHERE ");
        condition.write_query(self, out, Data::qualified_columns());
        if let Some(limit) = limit {
            let _ = write!(out, "\nLIMIT {}", limit);
        }
        out.push(';');
    }

    fn write_insert<'b, E, It>(&self, out: &mut String, entities: It, update: bool)
    where
        Self: Sized,
        E: Entity + 'b,
        It: IntoIterator<Item = &'b E>,
    {
        let mut rows = entities.into_iter().map(Entity::row_filtered).peekable();
        let Some(mut row) = rows.next() else {
            return;
        };
        out.push_str("INSERT INTO ");
        self.write_table_ref(out, E::table_ref(), false);
        out.push_str(" (");
        let columns = E::columns().iter();
        let single = rows.peek().is_none();
        if single {
            // Inserting a single row uses row_labeled to filter out Passive::NotSet columns
            separated_by(
                out,
                row.iter(),
                |out, v| self.write_identifier_quoted(out, v.0),
                ", ",
            );
        } else {
            // Inserting more rows will list all columns, Passive::NotSet columns will result in DEFAULT value
            separated_by(
                out,
                columns.clone(),
                |out, v| self.write_identifier_quoted(out, v.name()),
                ", ",
            );
        };
        out.push_str(") VALUES\n");
        let mut first_row = None;
        let mut separate = false;
        loop {
            if separate {
                out.push_str(",\n");
            }
            out.push('(');
            let mut fields = row.iter();
            let mut field = fields.next();
            separated_by(
                out,
                E::columns(),
                |out, col| {
                    if Some(col.name()) == field.map(|v| v.0) {
                        self.write_value(
                            out,
                            field
                                .map(|v| &v.1)
                                .expect("Column does not to have a value"),
                        );
                        field = fields.next();
                    } else if !single {
                        out.push_str("DEFAULT");
                    }
                },
                ", ",
            );
            out.push(')');
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
        out.push(';');
    }

    fn write_insert_update_fragment<'a, E, It>(&self, out: &mut String, columns: It)
    where
        Self: Sized,
        E: Entity,
        It: Iterator<Item = &'a ColumnDef>,
    {
        let pk = E::primary_key_def();
        if pk.len() == 0 {
            return;
        }
        out.push_str("\nON CONFLICT");
        if pk.len() > 0 {
            out.push_str(" (");
            separated_by(
                out,
                pk,
                |out, v| self.write_identifier_quoted(out, v.name()),
                ", ",
            );
            out.push(')');
        }
        out.push_str(" DO UPDATE SET\n");
        separated_by(
            out,
            columns.filter(|c| c.primary_key == PrimaryKeyType::None),
            |out, v| {
                self.write_identifier_quoted(out, v.name());
                out.push_str(" = EXCLUDED.");
                self.write_identifier_quoted(out, v.name());
            },
            ",\n",
        );
    }

    fn write_delete<E: Entity, Expr: Expression>(&self, out: &mut String, condition: &Expr)
    where
        Self: Sized,
    {
        out.push_str("DELETE FROM ");
        self.write_table_ref(out, E::table_ref(), false);
        out.push_str("\nWHERE ");
        condition.write_query(self, out, false);
        out.push(';');
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
