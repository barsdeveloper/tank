use crate::{Driver, Error, Executor, Query, Result};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub struct PreparedCache<D: Driver> {
    cache: RwLock<HashMap<Arc<str>, D::Prepared>>,
}

impl<D: Driver> PreparedCache<D> {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, query: &str) -> Option<Query<D::Prepared>> {
        self.cache
            .read()
            .await
            .get(query)
            .map(|v| Query::Prepared(v.clone()))
    }

    pub async fn as_prepared<'a, E: Executor<Driver = D>>(
        &self,
        executor: &mut E,
        query: &'a mut Query<D::Prepared>,
    ) -> Result<&'a mut Query<D::Prepared>> {
        if let Query::Raw(value) = query {
            let cache = self.cache.read().await;
            *query = if let Some(prepared) = cache.get(value) {
                let prepared = prepared.clone();
                drop(cache);
                prepared.into()
            } else {
                let prepared = executor.prepare(value.to_string()).await?;
                let Query::Prepared(prepared) = prepared else {
                    return Err(Error::msg(
                        "Prepared method is expected to return the Query::Prepared variatn",
                    ));
                };
                let mut cache = self.cache.write().await;
                cache.insert(value.clone(), prepared.clone());
                prepared.into()
            };
        }
        Ok(query)
    }
}
