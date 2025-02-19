use metadata::{ColumnDef, ColumnRef, DataType};
use regex::Regex;
use std::sync::LazyLock;

use crate::Backend;

pub trait WriteSql {
    fn identifier_regex(&self) -> &'static Regex {
        static IDENTIFIER_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap());
        &IDENTIFIER_REGEX
    }

    fn is_identifier(&self, name: &str) -> bool {
        self.identifier_regex().is_match(name)
    }

    fn write_sql<B: Backend>(&self, backend: &B, output: &mut String);
}

impl WriteSql for DataType {
    fn write_sql<B: Backend>(&self, backend: &B, output: &mut String) {
        // Not attempting to give a default, names are generally different for each DB
        backend.write_sql_type(self);
    }
}

impl WriteSql for ColumnDef {
    fn write_sql<B: Backend>(&self, backend: &B, output: &mut String) {
        output.push_str(&self.name);
        output.push_str(" ");
        self.data_type.write_sql(backend, output);
    }
}

impl WriteSql for ColumnRef {
    fn write_sql<B: Backend>(&self, backend: &B, output: &mut String) {
        if self.is_identifier(&self.name) {
            output.push_str(&self.name);
        } else {
            output.push_str(&format!("\"{}\"", self.name));
        }
    }
}
