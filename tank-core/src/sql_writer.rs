use crate::{
    BinaryOp, BinaryOpType, ColumnDef, ColumnRef, DataSet, Entity, Expression, Interval, Join,
    JoinType, Operand, PrimaryKeyType, TableRef, UnaryOp, UnaryOpType, Value,
    possibly_parenthesized, separated_by,
};
use std::fmt::Write;

macro_rules! write_integer {
    ($out:ident, $value:expr) => {{
        let mut buffer = itoa::Buffer::new();
        $out.push_str(buffer.format($value));
    }};
}
macro_rules! write_float {
    ($out:ident, $value:expr) => {{
        let mut buffer = ryu::Buffer::new();
        $out.push_str(buffer.format($value));
    }};
}

pub trait SqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter;

    fn write_table_ref(&self, out: &mut String, value: &TableRef) {
        if !value.alias.is_empty() {
            out.push_str(&value.alias);
        } else {
            if !value.schema.is_empty() {
                out.push_str(&value.schema);
                out.push('.');
            }
            out.push_str(&value.name);
        }
    }

    fn write_column_ref(&self, out: &mut String, value: &ColumnRef, qualify: bool) {
        if qualify && !value.table.is_empty() {
            if !value.schema.is_empty() {
                out.push_str(&value.schema);
                out.push('.');
            }
            out.push_str(&value.table);
            out.push('.');
        }
        out.push_str(&value.name);
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
                    let _ = write!(out, "({}, {})", precision, scale);
                }
            }
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
                out.push_str(", ");
                self.write_column_type(out, value);
                out.push(')');
            }
            _ => panic!("Unexpected tank::Value, cannot get the sql type"),
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
            | Value::Map(None, ..) => self.write_value_none(out),
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
            Value::Float32(Some(v), ..) => write_float!(out, *v),
            Value::Float64(Some(v), ..) => write_float!(out, *v),
            Value::Decimal(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Varchar(Some(v), ..) => self.write_value_string(out, v),
            Value::Blob(Some(v), ..) => {
                out.push('\'');
                v.iter().for_each(|b| {
                    let _ = write!(out, "\\x{:X}", b);
                });
                out.push('\'');
            }
            Value::Date(Some(v), ..) => {
                out.push('\'');
                let _ = write!(out, "{}", v);
                out.push('\'');
            }
            Value::Time(Some(v), ..) => {
                out.push('\'');
                let _ = write!(out, "{}", v);
                out.push('\'');
            }
            Value::Timestamp(Some(v), ..) => {
                out.push('\'');
                let _ = write!(out, "{}", v);
                out.push('\'');
            }
            Value::TimestampWithTimezone(Some(v), ..) => {
                out.push('\'');
                let _ = write!(out, "{}", v);
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
            Value::Map(Some(v), ..) => {
                out.push('{');
                separated_by(
                    out,
                    v.iter(),
                    |out, (k, v)| {
                        self.write_value(out, k);
                        out.push(':');
                        self.write_value(out, v);
                    },
                    ",",
                );
                out.push('}');
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
            }
        }
        out.push_str(&value[position..]);
        out.push('\'');
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
        const UNITS: &[(&str, i128)] = &[
            ("DAY", Interval::NANOS_IN_DAY),
            ("HOUR", Interval::NANOS_IN_SEC * 3600),
            ("MINUTE", Interval::NANOS_IN_SEC * 60),
            ("SECOND", Interval::NANOS_IN_SEC),
            ("MICROSECOND", 1_000),
            ("NANOSECOND", 1),
        ];
        for &(name, factor) in UNITS {
            if nanos % factor == 0 {
                write_unit!(out, nanos / factor, name);
                units += 1;
                break;
            }
        }
        if units > 1 {
            out.insert(quote_position, '\'');
            out.push('\'');
        }
    }

    fn expression_unary_op_precedence<'a>(&self, value: &UnaryOpType) -> i32 {
        match value {
            UnaryOpType::Negative => 1050,
            UnaryOpType::Not => 350,
        }
    }

    fn expression_binary_op_precedence<'a>(&self, value: &BinaryOpType) -> i32 {
        match value {
            BinaryOpType::Cast => 100,
            BinaryOpType::Or => 200,
            BinaryOpType::And => 300,
            BinaryOpType::Equal => 400,
            BinaryOpType::NotEqual => 400,
            BinaryOpType::Less => 400,
            BinaryOpType::Greater => 400,
            BinaryOpType::LessEqual => 400,
            BinaryOpType::GreaterEqual => 400,
            BinaryOpType::Is => 500,
            BinaryOpType::IsNot => 500,
            BinaryOpType::Like => 500,
            BinaryOpType::NotLike => 500,
            BinaryOpType::Regexp => 500,
            BinaryOpType::NotRegexp => 500,
            BinaryOpType::Glob => 500,
            BinaryOpType::NotGlob => 500,
            BinaryOpType::BitwiseOr => 600,
            BinaryOpType::BitwiseAnd => 700,
            BinaryOpType::ShiftLeft => 800,
            BinaryOpType::ShiftRight => 800,
            BinaryOpType::Subtraction => 900,
            BinaryOpType::Addition => 900,
            BinaryOpType::Multiplication => 1000,
            BinaryOpType::Division => 1000,
            BinaryOpType::Remainder => 1000,
            BinaryOpType::Indexing => 1100,
        }
    }

    fn write_expression_operand(&self, out: &mut String, value: &Operand, qualify_columns: bool) {
        let _ = match value {
            Operand::LitBool(v) => self.write_value_bool(out, *v),
            Operand::LitFloat(v) => write_float!(out, *v),
            Operand::LitIdent(v) => out.push_str(v),
            Operand::LitField(v) => {
                v.iter().fold('\0', |sep, segment| {
                    out.push(sep);
                    out.push_str(segment);
                    '.'
                });
            }
            Operand::LitInt(v) => write_integer!(out, *v),
            Operand::LitStr(v) => self.write_value_string(out, v),
            Operand::LitArray(v) => {
                out.push('[');
                separated_by(
                    out,
                    v.iter(),
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
                    args.iter(),
                    |out, v| {
                        v.write_query(self.as_dyn(), out, qualify_columns);
                    },
                    ",",
                );
                out.push(')');
            }
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
        out.push_str(E::table_ref().schema);
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
        out.push_str(E::table_ref().schema);
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
        self.write_table_ref(out, E::table_ref());
        out.push_str(" (\n");
        separated_by(
            out,
            E::columns_def().iter(),
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
                E::primary_key_def(),
                |out, v| out.push_str(v.name()),
                ", ",
            );
            out.push(')');
        }
        for unique in E::unique_defs() {
            if unique.len() > 1 {
                out.push_str(",\nUNIQUE (");
                separated_by(out, unique, |out, v| out.push_str(v.name()), ", ");
                out.push(')');
            }
        }
        out.push_str("\n)");
        out.push(';');
    }

    fn write_create_table_column_fragment(&self, out: &mut String, column: &ColumnDef) {
        out.push_str(&column.name());
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
        if !column.comment.is_empty() {
            self.write_create_table_column_fragment_comment(out, column);
        }
    }

    fn write_create_table_column_fragment_comment(&self, out: &mut String, column: &ColumnDef) {
        out.push_str(" COMMENT ");
        self.write_value_string(out, column.comment);
    }

    fn write_drop_table<E: Entity>(&self, out: &mut String, if_exists: bool)
    where
        Self: Sized,
    {
        out.push_str("DROP TABLE ");
        if if_exists {
            out.push_str("IF EXISTS ");
        }
        self.write_table_ref(out, E::table_ref());
        out.push(';');
    }

    fn write_select<'a, I, C, S, Expr>(
        &self,
        out: &mut String,
        columns: C,
        from: &S,
        condition: &Expr,
        limit: Option<u32>,
    ) where
        Self: Sized,
        I: Into<&'a dyn Expression>,
        C: Iterator<Item = I>,
        S: DataSet,
        Expr: Expression,
    {
        out.push_str("SELECT ");
        separated_by(
            out,
            columns,
            |out, col| {
                col.into().write_query(self, out, S::qualified_columns());
            },
            ", ",
        );
        out.push_str("\nFROM ");
        from.write_query(self, out);
        out.push_str("\nWHERE ");
        condition.write_query(self, out, S::qualified_columns());
        if let Some(limit) = limit {
            let _ = write!(out, "\nLIMIT {}", limit);
        }
        out.push(';');
    }

    fn write_insert<'b, E, It>(&self, out: &mut String, entities: It, replace: bool)
    where
        Self: Sized,
        E: Entity + 'b,
        It: Iterator<Item = &'b E>,
    {
        let mut rows = entities.map(Entity::row_filtered).peekable();
        let Some(mut row) = rows.next() else {
            return;
        };
        out.push_str("INSERT");
        if replace {
            out.push_str(" OR REPLACE");
        }
        out.push_str(" INTO ");
        self.write_table_ref(out, E::table_ref());
        out.push_str(" (");
        let columns = E::columns_def().iter().map(|v| v.name());
        let single = rows.peek().is_none();
        if single {
            // Inserting a single row uses row_labeled to filter out Passive::NotSet columns
            separated_by(out, row.iter().map(|v| v.0), |out, v| out.push_str(v), ", ");
        } else {
            // Inserting more rows will list all columns
            separated_by(out, columns.clone(), |out, v| out.push_str(v), ", ");
        };
        out.push_str(")\nVALUES");
        if single {
            out.push_str(" (");
            separated_by(
                out,
                row.into_iter().map(|v| v.1),
                |out, v| self.write_value(out, &v),
                ", ",
            );
            out.push(')');
        } else {
            let mut separate = false;
            loop {
                if separate {
                    out.push(',');
                }
                out.push_str("\n(");
                let mut fields = row.iter();
                let mut field = fields.next();
                {
                    let mut separate = false;
                    for name in columns.clone() {
                        if separate {
                            out.push_str(", ");
                        }
                        if Some(name) == field.map(|v| v.0) {
                            self.write_value(out, field.map(|v| &v.1).expect("Exists"));
                            field = fields.next();
                        } else {
                            out.push_str("DEFAULT");
                        }
                        separate = true;
                    }
                    out.push(')');
                }
                separate = true;
                if let Some(next) = rows.next() {
                    row = next;
                } else {
                    break;
                };
            }
        }
        out.push(';');
    }

    fn write_delete<E: Entity, Expr: Expression>(&self, out: &mut String, condition: &Expr)
    where
        Self: Sized,
    {
        out.push_str("DELETE FROM ");
        self.write_table_ref(out, E::table_ref());
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
