use std::pin::pin;

use async_stream::try_stream;
use tank_core::stream::{Stream, StreamExt};

use crate::ValueHolder;

pub(crate) fn row_to_tank_row(row: tokio_postgres::Row) -> tank_core::Row {
    (0..row.len())
        .map(|i| row.get::<_, ValueHolder>(i).0)
        .collect::<tank_core::Row>()
}

pub(crate) fn stream_postgres_row_to_tank_row<E>(
    stream: tokio_postgres::RowStream,
) -> impl Stream<Item = tank_core::Result<tank_core::RowLabeled>> {
    try_stream! {
        let mut stream = pin!(stream);
        if let Some(first) = stream.next().await {
            let labels = first?
                .columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect::<tank_core::RowNames>();
            while let Some(value) = stream.next().await {
                yield tank_core::RowLabeled {
                    labels: labels.clone(),
                    values: row_to_tank_row(value?).into(),
                }
                .into()
            }
        }
    }
}
