use std::sync::LazyLock;
use tank::{Entity, Executor};
use tokio::sync::Mutex;

pub async fn limits<E: Executor>(executor: &mut E) {
    #[derive(Entity)]
    struct Limits {
        boolean: bool,
        int8: i8,
        uint8: u8,
        int16: i16,
        uint16: u16,
        int32: i32,
        uint32: u32,
        int64: i64,
        #[cfg(not(feature = "disable-large-integers"))]
        uint64: u64,
        #[cfg(not(feature = "disable-large-integers"))]
        int128: i128,
        #[cfg(not(feature = "disable-large-integers"))]
        uint128: u128,
    }

    static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
    let _lock = MUTEX.lock().await;

    // Setup
    Limits::drop_table(executor, true, false)
        .await
        .expect("Failed to drop SimpleNullFields table");
    Limits::create_table(executor, true, true)
        .await
        .expect("Failed to create SimpleNullFields table");

    // Minimals
    Limits::delete_many(executor, &true)
        .await
        .expect("Failed to clear the Limits table");
    let entity = Limits {
        boolean: false,
        int8: -127,
        uint8: 0,
        int16: -32_768,
        uint16: 0,
        int32: -2_147_483_648,
        uint32: 0,
        int64: -9_223_372_036_854_775_808,
        #[cfg(not(feature = "disable-large-integers"))]
        uint64: 0,
        #[cfg(not(feature = "disable-large-integers"))]
        int128: -170_141_183_460_469_231_731_687_303_715_884_105_728,
        #[cfg(not(feature = "disable-large-integers"))]
        uint128: 0,
    };
    entity
        .save(executor)
        .await
        .expect("Filder to save minimals entity");
    let loaded = Limits::find_one(executor, &true)
        .await
        .expect("Failed to query simple 2")
        .expect("Failed to find simple 2");
    assert_eq!(loaded.boolean, false);
    assert_eq!(loaded.int8, -127);
    assert_eq!(loaded.uint8, 0);
    assert_eq!(loaded.int16, -32_768);
    assert_eq!(loaded.uint16, 0);
    assert_eq!(loaded.int32, -2_147_483_648);
    assert_eq!(loaded.uint32, 0);
    assert_eq!(loaded.int64, -9_223_372_036_854_775_808);
    #[cfg(not(feature = "disable-large-integers"))]
    assert_eq!(loaded.uint64, 0);
    #[cfg(not(feature = "disable-large-integers"))]
    assert_eq!(
        loaded.int128,
        -170_141_183_460_469_231_731_687_303_715_884_105_728
    );
    #[cfg(not(feature = "disable-large-integers"))]
    assert_eq!(loaded.uint128, 0);

    // Maximals
    Limits::delete_many(executor, &true)
        .await
        .expect("Failed to clear the Limits table");
    let entity = Limits {
        boolean: true,
        int8: 127,
        uint8: 255,
        int16: 32_767,
        uint16: 65_535,
        int32: 2_147_483_647,
        uint32: 4_294_967_295,
        int64: 9_223_372_036_854_775_807,
        #[cfg(not(feature = "disable-large-integers"))]
        uint64: 18_446_744_073_709_551_615,
        #[cfg(not(feature = "disable-large-integers"))]
        int128: 170_141_183_460_469_231_731_687_303_715_884_105_727,
        #[cfg(not(feature = "disable-large-integers"))]
        uint128: 340_282_366_920_938_463_463_374_607_431_768_211_455,
    };
    entity
        .save(executor)
        .await
        .expect("Failed to save maximals entity");
    let loaded = Limits::find_one(executor, &true)
        .await
        .expect("Failed to query simple 2")
        .expect("Failed to find simple 2");
    assert_eq!(loaded.boolean, true);
    assert_eq!(loaded.int8, 127);
    assert_eq!(loaded.uint8, 255);
    assert_eq!(loaded.int16, 32_767);
    assert_eq!(loaded.uint16, 65_535);
    assert_eq!(loaded.int32, 2_147_483_647);
    assert_eq!(loaded.uint32, 4_294_967_295);
    assert_eq!(loaded.int64, 9_223_372_036_854_775_807);
    #[cfg(not(feature = "disable-large-integers"))]
    assert_eq!(loaded.uint64, 18_446_744_073_709_551_615);
    #[cfg(not(feature = "disable-large-integers"))]
    assert_eq!(
        loaded.int128,
        170_141_183_460_469_231_731_687_303_715_884_105_727
    );
    #[cfg(not(feature = "disable-large-integers"))]
    assert_eq!(
        loaded.uint128,
        340_282_366_920_938_463_463_374_607_431_768_211_455
    );
}
