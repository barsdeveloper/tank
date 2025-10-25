use std::{pin::pin, str::FromStr, sync::LazyLock};
use tank::{
    Driver, Entity, Executor, Interval, QueryResult, SqlWriter,
    stream::{StreamExt, TryStreamExt},
};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Entity, Debug, PartialEq)]
struct Container {
    first: [[Interval; 2]; 1],
    second: [[[Uuid; 1]; 3]; 1],
}
static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub async fn arrays2<E: Executor>(executor: &mut E) {
    let _ = MUTEX.lock().await;

    let mut query = String::new();
    let writer = executor.driver().sql_writer();
    writer.write_drop_table::<Container>(&mut query, true);
    writer.write_create_table::<Container>(&mut query, true);
    let value = Container {
        first: [[
            Interval::from_years(500),
            Interval::from_micros(1) + Interval::from_months(4),
        ]],
        second: [[
            [Uuid::from_str("c1412337-cfce-444a-ac46-5e6edcc0bf23").unwrap()],
            [Uuid::from_str("00000000-0000-0000-0000-000000000000").unwrap()],
            [Uuid::from_str("9d7f0f5b-19d6-4298-a332-214fc85e2652").unwrap()],
        ]],
    };
    writer.write_insert(&mut query, &[value], false);
    writer.write_select(
        &mut query,
        Container::columns(),
        Container::table(),
        &true,
        Some(1),
    );
    let rows = pin!(executor.run(query.into()).try_filter_map(|v| async move {
        Ok(match v {
            QueryResult::Row(v) => Some(v),
            QueryResult::Affected(..) => None,
        })
    }));
    let rows = rows
        .map(|v| v.map(|row| Container::from_row(row)).flatten())
        .try_collect::<Vec<_>>()
        .await
        .expect("Coult not execute the query");
    assert_eq!(
        rows,
        [Container {
            first: [[
                Interval::from_years(500),
                Interval::from_micros(1) + Interval::from_months(4),
            ]],
            second: [[
                [Uuid::from_str("c1412337-cfce-444a-ac46-5e6edcc0bf23").unwrap()],
                [Uuid::from_str("00000000-0000-0000-0000-000000000000").unwrap()],
                [Uuid::from_str("9d7f0f5b-19d6-4298-a332-214fc85e2652").unwrap()],
            ]],
        }]
    )
}
