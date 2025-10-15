use crate::{PostgresTransaction, ValueWrap, postgres_type_to_value};
use std::{
    fmt::{self, Debug, Display},
    mem,
};
use tank_core::{AsValue, Error, Prepared, Result, future::Either};
use tokio_postgres::{Portal, Statement};

pub struct PostgresPrepared {
    pub(crate) statement: Statement,
    pub(crate) index: u64,
    pub(crate) value: Either<Vec<Option<ValueWrap>>, Portal>,
}

impl PostgresPrepared {
    pub(crate) fn new(statement: Statement) -> Self {
        Self {
            statement,
            index: 0,
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
        let types = self.statement.params();
        let mut params = mem::take(params);
        let mut i = 0;
        for param in &mut params {
            *param = Some(ValueWrap(
                mem::take(param)
                    .unwrap()
                    .0
                    .try_as(&postgres_type_to_value(&types[i]))?,
            ));
            i += 1;
        }
        let portal = transaction
            .0
            .bind_raw(&self.statement, params.into_iter().map(Option::unwrap))
            .await?;
        self.value = Either::Right(portal.clone());
        Ok(portal)
    }
    pub(crate) fn get_portal(&self) -> Option<Portal> {
        if let Either::Right(portal) = &self.value {
            Some(portal.clone())
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
        params.resize_with(self.statement.params().len(), || Default::default());
        params[index as usize] = Some(value.as_value().into());
        Ok(self)
    }
}

impl Display for PostgresPrepared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.statement.fmt(f)
    }
}
