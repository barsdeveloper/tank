use std::{borrow::Cow, sync::LazyLock};
use tank::{Entity, Executor};
use time::Time;
use tokio::sync::Mutex;
use uuid::Uuid;

pub async fn simple<E: Executor>(executor: &mut E) {
    #[derive(Entity)]
    struct SimpleFields {
        alpha: Option<u8>,
        bravo: Option<i32>,
        charlie: Option<i16>,
        delta: Option<u64>,
        echo: Option<Uuid>,
        foxtrot: Option<i128>,
        golf: Option<Time>,
        hotel: Option<Cow<'static, str>>,
        india: Box<Option<char>>,
    }

    static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
    let _lock = MUTEX.lock().await;

    // Setup
    SimpleFields::drop_table(executor, true, false)
        .await
        .expect("Failed to drop SimpleNullFields table");
    SimpleFields::create_table(executor, true, true)
        .await
        .expect("Failed to create SimpleNullFields table");

    // Simple 1
    SimpleFields::delete_many(executor, &true)
        .await
        .expect("Failed to clear the SimpleNullFields table");
    let entity = SimpleFields {
        alpha: None,
        bravo: 777.into(),
        charlie: (-2).into(),
        delta: 9876543210.into(),
        echo: None,
        foxtrot: i128::MAX.into(),
        golf: Time::from_hms(12, 0, 10).unwrap().into(),
        hotel: Some("Hello world!".into()),
        india: Box::new(None),
    };
    entity
        .save(executor)
        .await
        .expect("Failed to save simple 1");
    let entity = SimpleFields::find_one(executor, &true)
        .await
        .expect("Failed to query simple 1")
        .expect("Failed to find simple 1");
    assert_eq!(entity.alpha, None);
    assert_eq!(entity.bravo, Some(777));
    assert_eq!(entity.charlie, Some(-2));
    assert_eq!(entity.delta, Some(9876543210));
    assert_eq!(entity.echo, None);
    assert_eq!(
        entity.foxtrot,
        Some(170_141_183_460_469_231_731_687_303_715_884_105_727)
    );
    assert_eq!(entity.golf, Some(Time::from_hms(12, 0, 10).unwrap()));
    assert_eq!(entity.hotel, Some("Hello world!".into()));
    assert_eq!(*entity.india, None);

    // Simple 2
    SimpleFields::delete_many(executor, &true)
        .await
        .expect("Failed to clear the SimpleNullFields table");
    let entity = SimpleFields {
        alpha: 255.into(),
        bravo: None,
        charlie: None,
        delta: None,
        echo: Some(Uuid::parse_str("5e915574-bb30-4430-98cf-c5854f61fbbd").unwrap()),
        foxtrot: None,
        golf: None,
        hotel: None,
        india: Box::new(None),
    };
    entity
        .save(executor)
        .await
        .expect("Failed to save simple 2");
    let entity = SimpleFields::find_one(executor, &true)
        .await
        .expect("Failed to query simple 2")
        .expect("Failed to find simple 2");
    assert_eq!(entity.alpha, Some(255));
    assert_eq!(entity.bravo, None);
    assert_eq!(entity.charlie, None);
    assert_eq!(entity.delta, None);
    assert_eq!(
        entity.echo,
        Some(Uuid::parse_str("5e915574-bb30-4430-98cf-c5854f61fbbd").unwrap())
    );
    assert_eq!(entity.foxtrot, None);
    assert_eq!(entity.golf, None);
    assert_eq!(entity.hotel, None);
    assert_eq!(*entity.india, None);
}
