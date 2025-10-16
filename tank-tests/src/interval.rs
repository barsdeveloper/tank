use crate::silent_logs;
use std::{sync::LazyLock, time::Duration};
use tank::{Entity, Executor, Interval};
use tokio::sync::Mutex;

#[derive(Entity)]
struct Intervals {
    first: time::Duration,
    second: Interval,
    third: Duration,
}
static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub async fn interval<E: Executor>(executor: &mut E) {
    let _lock = MUTEX.lock().await;

    // Setup
    Intervals::drop_table(executor, true, false)
        .await
        .expect("Failed to drop Intervals table");
    Intervals::create_table(executor, true, true)
        .await
        .expect("Failed to create Intervals table");

    Intervals::insert_one(
        executor,
        &Intervals {
            first: Default::default(),
            second: Default::default(),
            third: Default::default(),
        },
    )
    .await
    .expect("Insert zero intervals failed");
    let value = Intervals::find_one(executor, &true)
        .await
        .expect("Failed to retrieve zero intervals")
        .expect("Missing zero interval row");
    assert_eq!(value.first, time::Duration::default());
    assert_eq!(value.second, Interval::default());
    assert_eq!(value.third, Duration::default());
    Intervals::delete_many(executor, &true)
        .await
        .expect("Could not delete the intervals");

    Intervals::insert_one(
        executor,
        &Intervals {
            first: time::Duration::minutes(1) + time::Duration::days(1),
            second: Interval::from_years(1_000),
            third: Duration::from_micros(1) + Duration::from_hours(6),
        },
    )
    .await
    .expect("Could not insert the interval");
    let value = Intervals::find_one(executor, &true)
        .await
        .expect("Could not retrieve the intervals row")
        .expect("There was no interval inserted in the table intervals");
    assert_eq!(value.first, time::Duration::minutes(1 + 24 * 60));
    assert_eq!(value.second, Interval::from_months(1_000 * 12));
    assert_eq!(value.third, Duration::from_micros(1 + 6 * 3600 * 1_000_000));
    Intervals::delete_many(executor, &true)
        .await
        .expect("Could not delete the intervals");

    Intervals::insert_one(
        executor,
        &Intervals {
            first: time::Duration::weeks(52) + time::Duration::hours(3),
            second: Interval::from_days(-11),
            third: Duration::from_micros(999_999_999),
        },
    )
    .await
    .expect("Insert large intervals failed");
    let mut value = Intervals::find_one(executor, &true)
        .await
        .expect("Failed to retrieve large intervals")
        .expect("Missing large interval row");
    assert_eq!(
        value.first,
        time::Duration::weeks(52) + time::Duration::hours(3)
    );
    assert_eq!(value.second, -Interval::from_days(11));
    assert_eq!(value.third, Duration::from_micros(999_999_999));
    value.third += Duration::from_micros(1);
    silent_logs! {
    value
        .save(executor)
        .await
        .expect_err("Should not be able to save a entity without a primary key");
    }
}
