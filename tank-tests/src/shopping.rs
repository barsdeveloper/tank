use rust_decimal::Decimal;
#[allow(unused_imports)]
use std::{str::FromStr, sync::Arc, sync::LazyLock};
use tank::{
    DataSet, Entity, Executor, FixedDecimal, cols, expr,
    stream::{StreamExt, TryStreamExt},
};
use time::{Date, PrimitiveDateTime};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Default, Debug, Entity)]
#[tank(schema = "shopping", primary_key = Self::name)]
struct Product {
    id: usize,
    name: String,
    price: FixedDecimal<8, 2>,
    desc: Option<String>,
    stock: Option<isize>,
    #[cfg(not(feature = "disable-lists"))]
    tags: Vec<String>,
}

#[derive(Debug, Entity)]
#[tank(schema = "shopping", primary_key = Self::id)]
struct User {
    id: Uuid,
    name: String,
    email: String,
    birthday: Date,
    #[cfg(not(feature = "disable-lists"))]
    preferences: Option<Arc<Vec<String>>>,
    registered: PrimitiveDateTime,
}

#[derive(Debug, Entity)]
#[tank(schema = "shopping")]
struct Cart {
    user: Uuid,
    #[tank(references = Product:: id)]
    product: usize,
    /// The price can stay locked once added to the shopping cart
    price: FixedDecimal<8, 2>,
    timestamp: PrimitiveDateTime,
}

static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub async fn shopping<E: Executor>(executor: &mut E) {
    let _lock = MUTEX.lock().await;

    let products = [
        Product {
            id: 1,
            name: "Rust-Proof Coffee Mug".into(),
            price: Decimal::from_str("12.99").unwrap().into(),
            desc: Some("Keeps your coffee warm and your compiler calm.".into()),
            stock: 42.into(),
            #[cfg(not(feature = "disable-lists"))]
            tags: vec!["kitchen".into(), "coffee".into(), "metal".into()].into(),
        },
        Product {
            id: 2,
            name: "Zero-Cost Abstraction Hoodie".into(),
            price: Decimal::from_str("49.95").unwrap().into(),
            desc: Some("For developers who think runtime overhead is a moral failure.".into()),
            stock: 10.into(),
            #[cfg(not(feature = "disable-lists"))]
            tags: vec!["clothing".into(), "nerdwear".into()].into(),
        },
        Product {
            id: 3,
            name: "Thread-Safe Notebook".into(),
            price: Decimal::from_str("7.50").unwrap().into(),
            desc: None,
            stock: 0.into(),
            #[cfg(not(feature = "disable-lists"))]
            tags: vec!["stationery".into()].into(),
        },
        Product {
            id: 4,
            name: "Async Teapot".into(),
            price: Decimal::from_str("25.00").unwrap().into(),
            desc: Some("Returns 418 on brew() call.".into()),
            stock: 3.into(),
            #[cfg(not(feature = "disable-lists"))]
            tags: vec!["kitchen".into(), "humor".into()].into(),
        },
    ];

    // Setup
    Product::drop_table(executor, true, true)
        .await
        .expect("Failed to drop product table");
    Product::create_table(executor, false, true)
        .await
        .expect("Failed to create the product table");

    Product::insert_many(executor, &products)
        .await
        .expect("Could not insert the products");
    let ordered_products = Product::table()
        .select(
            executor,
            cols!(Product::id, Product::name, Product::price ASC),
            &expr!(shopping.product.stock > 0),
            None,
        )
        .map_ok(Product::from_row)
        .map(Result::flatten)
        .try_collect::<Vec<_>>()
        .await
        .expect("Could not get the products ordered by increasing price");
    assert!(
        ordered_products.iter().map(|v| &v.name).eq([
            "Rust-Proof Coffee Mug",
            "Async Teapot",
            "Zero-Cost Abstraction Hoodie"
        ]
        .into_iter())
    );
}
