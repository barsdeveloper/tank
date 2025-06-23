use tank::{Connection, Entity, Passive};

pub async fn average<C: Connection>(connection: &mut C) {
    #[derive(Default, Entity)]
    struct Values {
        id: Passive<u64>,
        value: u32,
    }

    let result = Values::drop_table(connection, true).await;
    assert!(
        result.is_ok(),
        "Failed to Values::drop_table: {:?}",
        result.unwrap_err()
    );

    let result = Values::create_table(connection, false).await;
    assert!(
        result.is_ok(),
        "Failed to Values::create_table: {:?}",
        result.unwrap_err()
    );

    // avg(1 + .. + 785901) = 392951
    let values = (1..785901).map(|value| Values {
        value,
        ..Default::default()
    });
    for value in values {
        let result = value.save(connection).await;
        assert!(
            result.is_ok(),
            "Failed to save value: {:?}",
            result.unwrap_err()
        );
    }
}
