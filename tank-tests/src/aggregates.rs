use std::collections::BTreeSet;
use std::{pin::pin, sync::LazyLock};
use tank::AsValue;
use tank::{Connection, DataSet, Entity, Passive, RowLabeled, expr, stream::StreamExt};
use tokio::sync::Mutex;

static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
#[derive(Default, Entity)]
struct Values {
    id: Passive<u64>,
    /// This column contains the actual value
    value: u32,
}

pub async fn aggregates<C: Connection>(connection: &mut C) {
    let _lock = MUTEX.lock();

    // Setup
    Values::drop_table(connection, true, false)
        .await
        .expect("Failed to drop Values table");
    Values::create_table(connection, false, false)
        .await
        .expect("Failed to create Values table");

    // Insert
    // 1 + .. + 11745 = 68978385
    // avg(1, .., 11745) = 5873
    let mut values = (1..11746).map(|value| Values {
        id: value.into(),
        value: value as u32,
    });
    loop {
        let rows = values.by_ref().take(2000).collect::<Vec<_>>();
        if rows.is_empty() {
            break;
        }
        let result = Values::insert_many(connection, rows.iter()).await;
        assert!(
            result.is_ok(),
            "Failed to Values::insert_many: {:?}",
            result.unwrap_err()
        );
        let result = result.unwrap();
        assert_eq!(
            result.rows_affected,
            rows.len() as u64,
            "Values::insert_many should have affected {} rows",
            rows.len()
        );
    }

    // SELECT COUNT(*), SUM(value)
    {
        let mut stream = pin!(Values::table_ref().select(
            [expr!(COUNT(*)), expr!(SUM(Values::value))],
            connection,
            &true,
            None
        ));
        let count = stream.next().await;
        assert!(
            stream.next().await.is_none(),
            "The query is expected to return a single row"
        );
        let expected = (11745, 68978385);
        let actual = match count {
            Some(Ok(RowLabeled { values, .. })) => {
                let a = i128::try_from_value((*values)[0].clone());
                let b = i128::try_from_value((*values)[1].clone());
                match (a, b) {
                    (Ok(a), Ok(b)) => Some((a, b)),
                    (Err(e), _) => panic!("{}", e),
                    (_, Err(e)) => panic!("{}", e),
                }
            }
            _ => None,
        };
        assert_eq!(
            actual,
            Some(expected),
            "SELECT COUNT(*), SUM(value) is expected to return {:?}",
            expected
        );
    }

    // SELECT *
    {
        let cols = [expr!(*)];
        {
            let stream = pin!(Values::table_ref().select(&cols, connection, &true, None));
            let values = stream
                .map(|row| {
                    let row = row.expect("Error while retriever the row");
                    let i = row
                        .names()
                        .iter()
                        .enumerate()
                        .find_map(|(i, v)| if v == "value" { Some(i) } else { None })
                        .expect("Column `value` must be present");
                    u32::try_from_value(row.values[i].clone())
                        .expect("The result must convert back to u32")
                })
                .collect::<BTreeSet<_>>()
                .await;
            assert!(
                values.into_iter().eq((1..11746).into_iter()),
                "The result received from the db contains all the values that were inserted"
            );
        }
        let _cols = cols; // Can still use it afterwards because it was borrowed to select
    }
}
