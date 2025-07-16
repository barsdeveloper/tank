use std::{pin::pin, sync::LazyLock};
use tank::AsValue;
use tank::{Connection, DataSet, Entity, Passive, RowLabeled, expr, stream::StreamExt};
use tokio::sync::Mutex;

static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub async fn aggregate_functions<C: Connection>(connection: &mut C) {
    #[derive(Default, Entity)]
    struct Values {
        id: Passive<u64>,
        value: u32,
    }
    let _lock = MUTEX.lock();

    let result = Values::drop_table(connection, true, false).await;
    assert!(
        result.is_ok(),
        "Failed to Values::drop_table: {:?}",
        result.unwrap_err()
    );

    let result = Values::create_table(connection, false, false).await;
    assert!(
        result.is_ok(),
        "Failed to Values::create_table: {:?}",
        result.unwrap_err()
    );

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

    {
        let mut stream =
            pin!(Values::table_ref().select([expr!(COUNT(*))], connection, &true, None));
        let count = stream.next().await;
        assert!(
            match count {
                Some(Ok(RowLabeled { values, .. }))
                    if match i64::try_from_value((*values)[0].clone()) {
                        Ok(v) => v == 11745,
                        Err(_) => false,
                    } =>
                {
                    true
                }
                _ => false,
            },
            "COUNT(*) is expected to return 11745"
        );

        assert!(
            stream.next().await.is_none(),
            "COUNT(*) is expected to return a single row"
        );
    }

    for value in values {
        let result = value.save(connection).await;
        assert!(
            result.is_ok(),
            "Failed to save value: {:?}",
            result.unwrap_err()
        );
    }
}
