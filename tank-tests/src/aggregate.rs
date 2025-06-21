use tank::{Connection, Entity, Passive};

pub async fn average<C: Connection>(connection: &mut C) {
    #[derive(Entity)]
    struct Values {
        id: Passive<u64>,
        value: u32,
    }

    // avg(1 + .. + 785901) = 392951
    // (1..785901).map(f)
}
