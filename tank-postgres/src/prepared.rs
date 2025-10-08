use crate::{PostgresTransaction, ValueHolder};
use std::{
    fmt::{self, Debug, Display},
    mem,
};
use tank_core::{AsValue, Error, Prepared, Result, future::Either};
use tokio_postgres::{Portal, Statement};

pub struct PostgresPrepared {
    pub(crate) statement: Statement,
    pub(crate) index: u64,
    pub(crate) value: Either<Vec<Option<ValueHolder>>, Portal>,
}

impl PostgresPrepared {
    pub(crate) fn new(statement: Statement) -> Self {
        Self {
            statement,
            index: Default::default(),
            value: Either::Left(vec![]),
        }
    }
    pub(crate) fn is_complete(&self) -> bool {
        matches!(self.value, Either::Right(..))
    }
    pub(crate) async fn complete<'t>(
        &mut self,
        transaction: &mut PostgresTransaction<'t>,
    ) -> Result<Portal> {
        let Either::Left(params) = &mut self.value else {
            return Err(Error::msg("The prepared statement is already complete"));
        };
        if let Some(i) = params
            .iter()
            .enumerate()
            .find_map(|(i, v)| if v.is_none() { Some(i) } else { None })
        {
            return Err(Error::msg(format!("The parameter {} was not set", i)));
        }
        let portal = transaction
            .0
            .bind_raw(
                &self.statement,
                mem::take(params).into_iter().map(Option::unwrap),
            )
            .await?;
        self.value = Either::Right(portal.clone());
        Ok(portal)
    }
    pub(crate) fn get_portal(&self) -> Option<Portal> {
        if let Either::Right(portal) = &self.value {
            portal.clone().into()
        } else {
            None
        }
    }
}

impl Prepared for PostgresPrepared {
    fn bind<V: AsValue>(&mut self, value: V) -> Result<&mut Self> {
        self.bind_index(value, self.index)
    }
    fn bind_index<V: AsValue>(&mut self, value: V, index: u64) -> Result<&mut Self> {
        let Either::Left(params) = &mut self.value else {
            return Err(Error::msg("The prepared statement is already complete"));
        };
        params.reserve(self.statement.params().len());
        params[index as usize] = Some(value.as_value().into());
        Ok(self)
    }
}

impl Display for PostgresPrepared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.statement.fmt(f)
    }
}
