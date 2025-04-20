use crate::{
    BinaryOp, BinaryOpType, ColumnDef, Entity, Expression, Operand, UnaryOp, UnaryOpType, Value,
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

pub trait SqlWriter {
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

    fn sql_column_reference<'a>(&self, out: &'a mut String, value: &ColumnDef) -> &'a mut String {
        out.push_str(&value.full_name());
        out
    }

    fn expression_unary_op_precedence<'a>(&self, value: &UnaryOpType) -> i32 {
        match value {
            UnaryOpType::Negative => 950,
            UnaryOpType::Not => 250,
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
            BinaryOpType::BitwiseOr => 500,
            BinaryOpType::BitwiseAnd => 600,
            BinaryOpType::ShiftLeft => 700,
            BinaryOpType::ShiftRight => 700,
            BinaryOpType::Subtraction => 800,
            BinaryOpType::Addition => 800,
            BinaryOpType::Multiplication => 900,
            BinaryOpType::Division => 900,
            BinaryOpType::Remainder => 900,
            BinaryOpType::ArrayIndexing => 1000,
        }
    }

    fn sql_expression_operand<'a>(&self, out: &'a mut String, value: &Operand) -> &'a mut String {
        let _ = match value {
            Operand::LitBool(v) => write!(out, "{}", v),
            Operand::LitFloat(v) => write!(out, "{}", v),
            Operand::LitIdent(v) => write!(out, "{}", v),
            Operand::LitInt(v) => write!(out, "{}", v),
            Operand::LitStr(v) => write!(out, "'{}'", v),
            Operand::LitArray(v) => {
                out.push('[');
                v.iter().fold(false, |comma, v| {
                    if comma {
                        out.push_str(", ");
                    }
                    v.sql_write(self, out);
                    true
                });
                out.push(']');
                Ok(())
            }
            Operand::Column(v) => {
                out.push('"');
                self.sql_column_reference(out, v);
                out.push('"');
                Ok(())
            }
            Operand::Type(v) => {
                self.sql_type(out, v);
                Ok(())
            }
        };
        out
    }

    fn sql_expression_unary_op<'a, E: Expression>(
        &self,
        out: &'a mut String,
        value: &UnaryOp<E>,
    ) -> &'a mut String {
        let _ = match value.op {
            UnaryOpType::Negative => out.push('-'),
            UnaryOpType::Not => out.push_str("NOT "),
        };
        sql_possibly_parenthesized!(
            out,
            self.expression_unary_op_precedence(&value.op) < value.v.precedence(self),
            value.v.sql_write(self, out)
        );
        out
    }

    fn sql_expression_binary_op<'a, L: Expression, R: Expression>(
        &self,
        out: &'a mut String,
        value: &BinaryOp<L, R>,
    ) -> &'a mut String {
        let (prefix, infix, suffix) = match value.op {
            BinaryOpType::ArrayIndexing => ("", "[", "]"),
            BinaryOpType::Cast => ("CAST(", " AS ", ")"),
            BinaryOpType::Multiplication => ("", " * ", ""),
            BinaryOpType::Division => ("", " / ", ""),
            BinaryOpType::Remainder => ("", " % ", ""),
            BinaryOpType::Addition => ("", " + ", ""),
            BinaryOpType::Subtraction => ("", " - ", ""),
            BinaryOpType::ShiftLeft => ("", " << ", ""),
            BinaryOpType::ShiftRight => ("", " >> ", ""),
            BinaryOpType::BitwiseAnd => ("", " & ", ""),
            BinaryOpType::BitwiseOr => ("", " | ", ""),
            BinaryOpType::Equal => ("", " = ", ""),
            BinaryOpType::NotEqual => ("", " != ", ""),
            BinaryOpType::Less => ("", " < ", ""),
            BinaryOpType::LessEqual => ("", " <= ", ""),
            BinaryOpType::Greater => ("", " > ", ""),
            BinaryOpType::GreaterEqual => ("", " >= ", ""),
            BinaryOpType::And => ("", " AND ", ""),
            BinaryOpType::Or => ("", " OR ", ""),
        };
        let precedence = self.expression_binary_op_precedence(&value.op);
        out.push_str(prefix);
        sql_possibly_parenthesized!(
            out,
            value.lhs.precedence(self) > precedence,
            value.lhs.sql_write(self, out)
        );
        out.push_str(infix);
        // No parentheses needed because all known unary operators are prefix in this common SQL flavor
        value.rhs.sql_write(self, out);
        out.push_str(suffix);
        out
    }

    fn sql_create_table<'a, E: Entity>(
        &self,
        out: &'a mut String,
        if_not_exists: bool,
    ) -> &'a mut String {
        out.push_str("CREATE TABLE ");
        if if_not_exists {
            out.push_str("IF NOT EXISTS ");
        }
        out.push_str(E::table_name());
        out.push('(');
        let mut first = true;
        E::columns().iter().for_each(|c| {
            if !first {
                out.push_str(", ");
            }
            self.sql_create_table_column_fragment(out, c);
            first = false;
        });
        out.push(')');
        out
    }

    fn sql_create_table_column_fragment<'a>(
        &self,
        out: &'a mut String,
        column: &ColumnDef,
    ) -> &'a mut String {
        out.push_str(&column.name);
        out.push(' ');
        if !column.column_type.is_empty() {
            out.push_str(&column.column_type);
        } else {
            self.sql_type(out, &column.value);
        }
        if !column.nullable {
            out.push_str(" NOT NULL");
        }
        out
    }

    fn sql_drop_table<E: Entity>(&self, query: &mut String, if_exists: bool) {
        query.push_str("DROP TABLE ");
        if if_exists {
            query.push_str("IF EXISTS ");
        }
        query.push_str(E::table_name());
    }

    fn sql_select<'a, E: Entity>(
        &self,
        out: &'a mut String,
        condition: &[Value],
        limit: u32,
    ) -> &'a mut String {
        out.push_str("SELECT * FROM ");
        out.push_str(E::table_name());
        out.push_str(" WHERE ");
        out
    }
}
