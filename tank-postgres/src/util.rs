use crate::ValueHolder;
use async_stream::try_stream;
use std::pin::pin;
use tank_core::{
    Error, RowLabeled, RowNames,
    stream::{Stream, StreamExt},
};

pub(crate) fn row_to_tank_row(row: tokio_postgres::Row) -> tank_core::Result<tank_core::Row> {
    (0..row.len())
        .map(|i| match row.try_get::<_, ValueHolder>(i) {
            Ok(v) => Ok(v.0),
            Err(..) => {
                let col = &row.columns()[i];
                Err(Error::msg(format!(
                    "Could not deserialize column {} `{}`: {}",
                    i,
                    col.name(),
                    col.type_()
                )))
            }
        })
        .collect::<tank_core::Result<tank_core::Row>>()
}

pub(crate) fn stream_postgres_row_as_tank_row<V, R>(
    stream: impl AsyncFnOnce() -> tank_core::Result<V>,
) -> impl Stream<Item = tank_core::Result<R>>
where
    V: Stream<Item = Result<tokio_postgres::Row, tokio_postgres::Error>>,
    R: From<RowLabeled>,
{
    try_stream! {
        let stream = stream().await?;
        let mut stream = pin!(stream);
        let mut labels: Option<RowNames> = None;
        while let Some(row) = stream.next().await.transpose()? {
            let labels = labels.get_or_insert_with(|| {
                row.columns().iter().map(|c| c.name().to_string()).collect()
            });
            yield RowLabeled {
                labels: labels.clone(),
                values: row_to_tank_row(row)?.into(),
            }.into();
        }
    }
}
