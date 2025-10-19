use std::sync::LazyLock;
use tank::{Entity, Executor};
use tokio::sync::Mutex;

#[derive(Entity)]
struct First {}
static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub async fn full1<E: Executor>(executor: &mut E) {}
