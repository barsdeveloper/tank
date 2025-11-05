use std::fmt::{self, Display};
use tank_core::{AsValue, Prepared, Result};

#[derive(Debug)]
pub struct YourDBPrepared {
    pub(crate) index: u64,
}

impl YourDBPrepared {
    pub(crate) fn new() -> Self {
        Self { index: 1 }
    }
}

impl Prepared for YourDBPrepared {
    fn bind<V: AsValue>(&mut self, value: V) -> Result<&mut Self> {
        let index = self.index;
        self.index += 1;
        self.bind_index(value, index)
    }

    fn bind_index<V: tank_core::AsValue>(
        &mut self,
        value: V,
        index: u64,
    ) -> tank_core::Result<&mut Self> {
        Ok(self)
    }
}

impl Display for YourDBPrepared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("YourDBPrepared")
    }
}
