use crate::ValueWrap;
use mysql_async::Statement;
use std::{
    fmt::{self, Display},
    mem,
};
use tank_core::{AsValue, Prepared, Result, Value};

#[derive(Debug)]
pub struct MySQLPrepared {
    pub(crate) statement: Statement,
    pub(crate) params: Vec<Value>,
    pub(crate) index: u64,
}

impl MySQLPrepared {
    pub(crate) fn new(statement: Statement) -> Self {
        Self {
            statement,
            params: Vec::new(),
            index: 0,
        }
    }
    pub(crate) fn take_params(&mut self) -> Result<mysql_async::Params> {
        Ok(mysql_async::Params::Positional(
            mem::take(&mut self.params)
                .into_iter()
                .map(|v| ValueWrap(v).try_into())
                .collect::<Result<_>>()?,
        ))
    }
}

impl Prepared for MySQLPrepared {
    fn bind(&mut self, value: impl AsValue) -> Result<&mut Self> {
        let index = self.index;
        self.index += 1;
        self.bind_index(value, index)
    }

    fn bind_index(&mut self, value: impl AsValue, index: u64) -> Result<&mut Self> {
        if self.params.is_empty() {
            self.params.reserve(self.statement.num_params() as _);
        }
        self.params[index as usize] = value.into();
        Ok(self)
    }
}

impl Display for MySQLPrepared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MySQLPrepared")
    }
}
