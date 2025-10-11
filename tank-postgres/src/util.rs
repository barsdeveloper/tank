use crate::ValueHolder;
use async_stream::try_stream;
use postgres_types::{FromSql, Type};
use std::pin::pin;
use tank_core::{
    Value,
    stream::{Stream, StreamExt},
};
use tokio_postgres::SimpleQueryMessage;

pub(crate) fn simple_query_row_to_tank_row(
    row: tokio_postgres::SimpleQueryRow,
) -> tank_core::Result<tank_core::Row> {
    (0..row.len())
        .map(|i| match row.try_get(i) {
            Ok(Some(v)) => ValueHolder::from_sql(&Type::UNKNOWN, v.as_bytes())
                .map(|v| v.0)
                .map_err(|e| tank_core::Error::msg(format!("{:#}", e))),
            Ok(None) => Ok(Value::Null),
            Err(..) => {
                let col = &row.columns()[i];
                Err(tank_core::Error::msg(format!(
                    "Could not deserialize column {} `{}`",
                    i,
                    col.name(),
                )))
            }
        })
        .collect::<tank_core::Result<tank_core::Row>>()
}

pub(crate) fn row_to_tank_row(row: tokio_postgres::Row) -> tank_core::Result<tank_core::Row> {
    (0..row.len())
        .map(|i| match row.try_get::<_, ValueHolder>(i) {
            Ok(v) => Ok(v.0),
            Err(..) => {
                let col = &row.columns()[i];
                Err(tank_core::Error::msg(format!(
                    "Could not deserialize column {} `{}`: {}",
                    i,
                    col.name(),
                    col.type_()
                )))
            }
        })
        .collect::<tank_core::Result<tank_core::Row>>()
}

pub(crate) fn stream_postgres_row_to_tank_row<V, R>(
    stream: impl AsyncFnOnce() -> tank_core::Result<V>,
) -> impl Stream<Item = tank_core::Result<R>>
where
    V: Stream<Item = Result<tokio_postgres::Row, tokio_postgres::Error>>,
    R: From<tank_core::RowLabeled>,
{
    try_stream! {
        let stream = stream().await?;
        let mut stream = pin!(stream);
        let mut labels: Option<tank_core::RowNames> = None;
        while let Some(row) = stream.next().await.transpose()? {
            let labels = labels.get_or_insert_with(|| {
                row.columns().iter().map(|c| c.name().to_string()).collect()
            });
            yield tank_core::RowLabeled {
                labels: labels.clone(),
                values: row_to_tank_row(row)?.into(),
            }.into();
        }
    }
}

pub(crate) fn stream_postgres_simple_query_message_to_tank_query_result<V, R>(
    stream: impl AsyncFnOnce() -> tank_core::Result<V>,
) -> impl Stream<Item = tank_core::Result<R>>
where
    V: Stream<Item = Result<tokio_postgres::SimpleQueryMessage, tokio_postgres::Error>>,
    R: From<tank_core::QueryResult>,
{
    try_stream! {
        let stream = stream().await?;
        let mut stream = pin!(stream);
        let mut labels: Option<tank_core::RowNames> = None;
        while let Some(entry) = stream.next().await.transpose()? {
            match entry {
                SimpleQueryMessage::Row(row) => {
                    yield tank_core::QueryResult::RowLabeled(tank_core::RowLabeled {
                        labels: labels.as_ref().filter(|v| v.len() == row.len()).ok_or(
                            tank_core::Error::msg(
                                "Row columns names does not match the row currently received",
                            ),
                        )?.clone(),
                        values: simple_query_row_to_tank_row(row)?,
                    })
                    .into();
                }
                SimpleQueryMessage::CommandComplete(rows_affected) => {
                    yield tank_core::QueryResult::Affected(tank_core::RowsAffected {
                        rows_affected,
                        ..Default::default()
                    })
                    .into();
                }
                SimpleQueryMessage::RowDescription(columns) => {
                    labels = Some(
                        columns
                            .into_iter()
                            .map(|col| col.name().into())
                            .collect::<tank_core::RowNames>(),
                    );
                }
                _ => {}
            }
        }
    }
}
