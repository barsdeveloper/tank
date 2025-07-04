use crate::{
    BinaryOp, BinaryOpType, ColumnDef, ColumnRef, DataSet, Entity, Expression, Interval, Join,
    JoinType, Operand, PrimaryKeyType, TableRef, UnaryOp, UnaryOpType, Value,
};
use std::fmt::Write;

macro_rules! sql_possibly_parenthesized {
    ($out:ident, $cond:expr, $v:expr) => {
        if $cond {
            $out.push('(');
            $v;
            $out.push(')');
        } else {
            $v;
        }
    };
}

pub fn separated_by<T, F>(out: &mut String, it: impl Iterator<Item = T>, mut f: F, separator: &str)
where
    F: FnMut(&mut String, &T),
{
    it.fold(usize::MAX, |mut len, v| {
        if len < out.len() {
            out.push_str(separator);
        }
        len = out.len();
        f(out, &v);
        len
    });
}

pub trait SqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter;

    fn sql_type<'a>(&self, out: &'a mut String, value: &Value) -> &'a mut String {
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
                self.sql_type(out, inner);
                let _ = write!(out, "[{}]", size);
            }
            Value::List(.., inner) => {
                self.sql_type(out, inner);
                out.push_str("[]");
            }
            Value::Map(.., key, value) => {
                out.push_str("MAP(");
                self.sql_type(out, key);
                out.push_str(", ");
                self.sql_type(out, value);
                out.push(')');
            }
            _ => panic!("Unexpected tank::Value, cannot get the sql type"),
        };
        out
    }

    fn sql_value<'a>(&self, out: &'a mut String, value: &Value) -> &'a mut String {
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
            | Value::Map(None, ..) => out.push_str("NULL"),
            Value::Boolean(Some(v), ..) => out.push_str(["false", "true"][*v as usize]),
            Value::Int8(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Int16(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Int32(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Int64(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Int128(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::UInt8(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::UInt16(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::UInt32(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::UInt64(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::UInt128(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Float32(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Float64(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Decimal(Some(v), ..) => drop(write!(out, "{}", v)),
            Value::Varchar(Some(v), ..) => drop(write!(
                out,
                "'{}'",
                v.replace('\'', self.sql_escaped_apostrophe())
            )),
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
            Value::Interval(Some(v), ..) => {
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
                if v.months != 0 {
                    if v.months % 12 == 0 {
                        write_unit!(out, v.months / 12, "YEAR");
                        units += 1;
                    } else {
                        write_unit!(out, v.months, "MONTH");
                        units += 1;
                    }
                }
                let nanos = v.nanos + v.days as i128 * Interval::NANOS_IN_DAY;
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
                        self.sql_value(out, v);
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
                        self.sql_value(out, k);
                        out.push(':');
                        self.sql_value(out, v);
                    },
                    ",",
                );
                out.push('}');
            }
        };
        out
    }

    fn sql_escaped_apostrophe(&self) -> &'static str {
        "''"
    }

    fn sql_table_ref<'a>(&self, out: &'a mut String, value: &TableRef) -> &'a mut String {
        out.push_str(&value.full_name());
        out
    }

    fn sql_column_ref<'a>(
        &self,
        out: &'a mut String,
        value: &ColumnRef,
        qualify: bool,
    ) -> &'a mut String {
        if qualify && !value.table.is_empty() {
            if !value.schema.is_empty() {
                out.push_str(&value.schema);
                out.push('.');
            }
            out.push_str(&value.table);
            out.push('.');
        }
        out.push_str(&value.name);
        out
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
            BinaryOpType::NotRegexpr => 500,
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

    fn sql_expression_operand<'a>(
        &self,
        out: &'a mut String,
        value: &Operand,
        qualify_columns: bool,
    ) -> &'a mut String {
        let _ = match value {
            Operand::LitBool(v) => write!(out, "{}", v),
            Operand::LitFloat(v) => write!(out, "{}", v),
            Operand::LitIdent(v) => write!(out, "{}", v),
            Operand::LitField(v) => Ok(out.push_str(&v.join("."))),
            Operand::LitInt(v) => write!(out, "{}", v),
            Operand::LitStr(v) => write!(out, "'{}'", v),
            Operand::LitArray(v) => {
                out.push('[');
                separated_by(
                    out,
                    v.iter(),
                    |out, v| {
                        v.sql_write(self.as_dyn(), out, qualify_columns);
                    },
                    ", ",
                );
                out.push(']');
                Ok(())
            }
            Operand::Null => Ok(out.push_str("NULL")),
            Operand::Column(v) => {
                self.sql_column_ref(out, v, qualify_columns);
                Ok(())
            }
            Operand::Type(v) => {
                self.sql_type(out, v);
                Ok(())
            }
            Operand::Variable(v) => {
                self.sql_value(out, v);
                Ok(())
            }
            Operand::Call(f, args) => {
                out.push_str(f);
                out.push('(');
                separated_by(
                    out,
                    args.iter(),
                    |out, v| {
                        v.sql_write(self.as_dyn(), out, qualify_columns);
                    },
                    ",",
                );
                out.push(')');
                Ok(())
            }
        };
        out
    }

    fn sql_expression_unary_op<'a>(
        &self,
        out: &'a mut String,
        value: &UnaryOp<&dyn Expression>,
        qualify_columns: bool,
    ) -> &'a mut String {
        let _ = match value.op {
            UnaryOpType::Negative => out.push('-'),
            UnaryOpType::Not => out.push_str("NOT "),
        };
        sql_possibly_parenthesized!(
            out,
            value.v.precedence(self.as_dyn()) <= self.expression_unary_op_precedence(&value.op),
            value.v.sql_write(self.as_dyn(), out, qualify_columns)
        );
        out
    }

    fn sql_expression_binary_op<'a>(
        &self,
        out: &'a mut String,
        value: &BinaryOp<&dyn Expression, &dyn Expression>,
        qualify_columns: bool,
    ) -> &'a mut String {
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
            BinaryOpType::NotRegexpr => ("", " NOT REGEXP ", "", false, false),
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
        sql_possibly_parenthesized!(
            out,
            !lhs_parenthesized && value.lhs.precedence(self.as_dyn()) < precedence,
            value.lhs.sql_write(self.as_dyn(), out, qualify_columns)
        );
        out.push_str(infix);
        sql_possibly_parenthesized!(
            out,
            !rhs_parenthesized && value.rhs.precedence(self.as_dyn()) <= precedence,
            value.rhs.sql_write(self.as_dyn(), out, qualify_columns)
        );
        out.push_str(suffix);
        out
    }

    fn sql_join_type<'a>(&self, out: &'a mut String, join_type: &JoinType) -> &'a mut String {
        out.push_str(match &join_type {
            JoinType::Inner => "INNER JOIN",
            JoinType::Outer => "OUTER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Cross => "CROSS",
            JoinType::Natural => "NATURAL JOIN",
        });
        out
    }

    fn sql_join<'a>(
        &self,
        out: &'a mut String,
        join: &Join<&dyn DataSet, &dyn DataSet, &dyn Expression>,
    ) -> &'a mut String {
        join.lhs.sql_write(self.as_dyn(), out);
        out.push(' ');
        self.sql_join_type(out, &join.join);
        out.push(' ');
        join.rhs.sql_write(self.as_dyn(), out);
        if let Some(on) = &join.on {
            out.push_str(" ON ");
            on.sql_write(self.as_dyn(), out, true);
        }
        out
    }

    fn sql_create_schema<'a, E>(&self, out: &'a mut String, if_not_exists: bool) -> &'a mut String
    where
        Self: Sized,
        E: Entity,
    {
        out.push_str("CREATE SCHEMA ");
        if if_not_exists {
            out.push_str("IF NOT EXISTS ");
        }
        out.push_str(E::table_ref().schema);
        out
    }

    fn sql_drop_schema<'a, E>(&self, out: &'a mut String, if_exists: bool) -> &'a mut String
    where
        Self: Sized,
        E: Entity,
    {
        out.push_str("DROP SCHEMA ");
        if if_exists {
            out.push_str("IF EXISTS ");
        }
        out.push_str(E::table_ref().schema);
        out
    }

    fn sql_create_table<'a, E>(&self, out: &'a mut String, if_not_exists: bool) -> &'a mut String
    where
        Self: Sized,
        E: Entity,
    {
        out.push_str("CREATE TABLE ");
        if if_not_exists {
            out.push_str("IF NOT EXISTS ");
        }
        self.sql_table_ref(out, E::table_ref());
        out.push_str(" (\n");
        separated_by(
            out,
            E::columns_def().iter(),
            |out, v| {
                self.sql_create_table_column_fragment(out, v);
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
        out
    }

    fn sql_autoincrement_fragment<'a>(&self, out: &'a mut String) -> &'a mut String {
        out.push_str("AUTOINCREMENT");
        out
    }

    fn sql_create_table_column_fragment<'a>(
        &self,
        out: &'a mut String,
        column: &ColumnDef,
    ) -> &'a mut String {
        out.push_str(&column.name());
        out.push(' ');
        if !column.column_type.is_empty() {
            out.push_str(&column.column_type);
        } else {
            self.sql_type(out, &column.value);
        }
        if !column.nullable && column.primary_key == PrimaryKeyType::None {
            out.push_str(" NOT NULL");
        }
        if let Some(default) = &column.default {
            out.push_str(" DEFAULT ");
            default.sql_write(self.as_dyn(), out, true);
        }
        if column.auto_increment {
            out.push(' ');
            self.sql_autoincrement_fragment(out);
        }
        if column.primary_key == PrimaryKeyType::PrimaryKey {
            // Composite primary key will be printed elsewhere
            out.push_str(" PRIMARY KEY");
        }
        if column.unique && column.primary_key != PrimaryKeyType::PrimaryKey {
            out.push_str(" UNIQUE");
        }
        if !column.comment.is_empty() {
            self.sql_create_table_column_fragment_comment(out, column);
        }
        out
    }

    fn sql_create_table_column_fragment_comment<'a>(
        &self,
        out: &'a mut String,
        column: &ColumnDef,
    ) -> &'a mut String {
        out.push_str(" COMMENT ");
        self.sql_value(out, &Value::Varchar(Some(column.comment.to_string())));
        out
    }

    fn sql_drop_table<'a, E: Entity>(&self, out: &'a mut String, if_exists: bool) -> &'a mut String
    where
        Self: Sized,
    {
        out.push_str("DROP TABLE ");
        if if_exists {
            out.push_str("IF EXISTS ");
        }
        self.sql_table_ref(out, E::table_ref());
        out
    }

    fn sql_select<'a, E: Entity, D: DataSet, Expr: Expression>(
        &self,
        out: &'a mut String,
        from: &D,
        condition: &Expr,
        limit: Option<u32>,
    ) -> &'a mut String
    where
        Self: Sized,
    {
        out.push_str("SELECT ");
        separated_by(
            out,
            E::columns_def().iter(),
            |out, col| {
                self.sql_column_ref(out, (*col).into(), D::qualified_columns());
            },
            ", ",
        );
        out.push_str("\nFROM ");
        from.sql_write(self, out);
        out.push_str("\nWHERE ");
        condition.sql_write(self, out, D::qualified_columns());
        if let Some(limit) = limit {
            let _ = write!(out, "\nLIMIT {}", limit);
        }
        out
    }

    fn sql_insert<'a, 'b, E, It>(
        &self,
        out: &'a mut String,
        entities: It,
        replace: bool,
    ) -> &'a mut String
    where
        Self: Sized,
        E: Entity + 'b,
        It: ExactSizeIterator<Item = &'b E>,
    {
        out.push_str("INSERT");
        if replace {
            out.push_str(" OR REPLACE");
        }
        out.push_str(" INTO ");
        self.sql_table_ref(out, E::table_ref());
        out.push_str(" (");
        let solo = entities.len() == 1;
        let mut entities = entities.peekable();
        if solo {
            separated_by(
                out,
                // Inserting one row uses row_labeled to filter out Passive::NotSet columns
                entities
                    .peek()
                    .expect("There is one element")
                    .row_filtered()
                    .names()
                    .iter(),
                |out, v| out.push_str(v),
                ", ",
            );
        } else {
            separated_by(
                out,
                // Inserting more rows will list all columns
                E::columns_def().iter().map(|v| v.name()),
                |out, v| out.push_str(v),
                ", ",
            );
        };
        out.push_str(")\nVALUES ");
        for entity in entities {
            let row = if solo {
                entity.row_filtered().values
            } else {
                entity.row_full()
            };
            out.push('(');
            separated_by(
                out,
                row.iter(),
                |out, v| {
                    self.sql_value(out, v);
                },
                ", ",
            );
            out.push(')');
        }
        out
    }

    fn sql_delete<'a, E: Entity, Expr: Expression>(
        &self,
        out: &'a mut String,
        condition: &Expr,
    ) -> &'a mut String
    where
        Self: Sized,
    {
        out.push_str("DELETE FROM ");
        self.sql_table_ref(out, E::table_ref());
        out.push_str("\nWHERE ");
        condition.sql_write(self, out, false);
        out
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
