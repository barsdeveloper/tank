use std::{collections::HashSet, ops::Deref, sync::LazyLock};
use tank::{
    Connection, DataSet, Entity, Expression, Passive, RowLabeled, Value, expr, join,
    stream::{StreamExt, TryStreamExt},
};
use tokio::sync::Mutex;
use uuid::Uuid;

static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Entity, Debug, Clone)]
#[tank(schema = "testing", name = "authors")]
pub struct Author {
    #[tank(primary_key, name = "author_id")]
    pub id: Passive<Uuid>,
    pub name: String,
    pub country: String,
}

#[derive(Entity, Debug, Clone)]
#[tank(schema = "testing", name = "books")]
pub struct Book {
    #[tank(primary_key)]
    pub isbn: [u8; 13],
    pub title: String,
    /// Main author
    pub author: Uuid,
    pub co_author: Option<Uuid>,
    pub year: i32,
}

pub async fn books<C: Connection>(connection: &mut C) {
    let _lock = MUTEX.lock().await;

    // Setup
    Book::drop_table(connection, true, false)
        .await
        .expect("Failed to drop Book table");
    Author::drop_table(connection, true, false)
        .await
        .expect("Failed to drop Author table");
    Author::create_table(connection, false, true)
        .await
        .expect("Failed to create Author table");
    Book::create_table(connection, false, true)
        .await
        .expect("Failed to create Book table");

    // Author objects
    let authors = vec![
        Author {
            id: Uuid::parse_str("f938f818-0a40-4ce3-8fbc-259ac252a1b5")
                .unwrap()
                .into(),
            name: "J.R.R. Tolkien".into(),
            country: "UK".into(),
        },
        Author {
            id: Uuid::parse_str("a73bc06a-ff89-44b9-a62f-416ebe976285")
                .unwrap()
                .into(),
            name: "George R.R. Martin".into(),
            country: "USA".into(),
        },
        Author {
            id: Uuid::parse_str("6b2f56a1-316d-42b9-a8ba-baca42c5416c")
                .unwrap()
                .into(),
            name: "Dmitrij Gluchovskij".into(),
            country: "Russia".into(),
        },
        Author {
            id: Uuid::parse_str("d3d3d3d3-d3d3-d3d3-d3d3-d3d3d3d3d3d3")
                .unwrap()
                .into(),
            name: "Linus Torvalds".into(),
            country: "Finland".into(),
        },
    ];
    let tolkien_id = authors[0].id.clone().unwrap();
    let martin_id = authors[1].id.clone().unwrap();
    let gluchovskij_id = authors[2].id.clone().unwrap();

    // Book objects
    let books = vec![
        Book {
            isbn: [9, 7, 8, 0, 0, 0, 7, 4, 4, 0, 8, 3, 2],
            title: "The Hobbit".into(),
            author: tolkien_id,
            co_author: None,
            year: 1937,
        },
        Book {
            isbn: [9, 7, 8, 0, 0, 0, 2, 2, 4, 5, 8, 4, 5],
            title: "A Game of Thrones (A Song of Ice and Fire book 1) ".into(),
            author: martin_id,
            co_author: None,
            year: 1996,
        },
        Book {
            isbn: [9, 7, 8, 5, 1, 7, 0, 5, 9, 6, 7, 8, 2],
            title: "Metro 2033".into(),
            author: gluchovskij_id,
            co_author: None,
            year: 2002,
        },
        Book {
            isbn: [9, 7, 8, 0, 0, 2, 3, 4, 5, 6, 7, 8, 9],
            title: "Mordor 2033".into(),
            author: tolkien_id,
            co_author: gluchovskij_id.into(),
            year: 2003,
        },
    ];

    // Insert
    let result = Author::insert_many(connection, authors.iter())
        .await
        .expect("Failed to insert authors");
    assert_eq!(result.rows_affected, 4);
    let result = Book::insert_many(connection, books.iter())
        .await
        .expect("Failed to insert books");
    assert_eq!(result.rows_affected, 4);

    // Get books before 2000
    let result = join!(Book B JOIN Author A ON B.author_id == A.author_id)
        .select(
            &[expr!(B.title), expr!(A.name)],
            connection,
            &expr!(B.publication_year < 2000),
            None,
        )
        .try_collect::<Vec<RowLabeled>>()
        .await
        .expect("Failed to query books and authors joined")
        .into_iter()
        .map(|row| {
            let mut iter = row.values.into_iter();
            (
                match iter.next().unwrap() {
                    Value::Varchar(Some(v)) => v,
                    _ => panic!("Expected first value to be non null varchar"),
                },
                match iter.next().unwrap() {
                    Value::Varchar(Some(v)) => v,
                    _ => panic!("Expected second value to be non null varchar"),
                },
            )
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        result,
        HashSet::from_iter([
            ("A Game of Thrones".into(), "George R.R. Martin".into()),
            ("The Hobbit".into(), "J.R.R. Tolkien".into()),
        ])
    );

    // Get all books with their authors
    let result = join!(Book B LEFT JOIN Author A1 ON B.author == A1.name JOIN Author A2 ON B.co_author == A2.name)
        .select(
            &[&expr!(B.title) as &dyn Expression, &expr!(A1.name as author), &expr!(A2.name as co_author)],
            connection,
            &true,
            None,
        )
        .try_collect::<Vec<RowLabeled>>()
        .await
        .expect("Failed to query books and authors joined")
        .into_iter()
        .map(|row| {
            let mut iter = row.values.into_iter();
            (
                match iter.next().unwrap()  {
                    Value::Varchar(Some(v)) => v,
                    _ => panic!("Expected first value to be non null varchar"),
                },
                match iter.next().unwrap()  {
                    Value::Varchar(Some(v)) => v,
                    _ => panic!("Expected second value to be non null varchar"),
                },
                match iter.next().unwrap()  {
                    Value::Varchar(Some(v)) => Some(v),
                    Value::Varchar(None) => None,
                    _ => panic!("Expected third value to be non null varchar"),
                },
            )
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        result,
        HashSet::from_iter([
            (
                "A Game of Thrones".into(),
                "George R.R. Martin".into(),
                None
            ),
            ("Metro 2033".into(), "Dmitrij Gluchovskij".into(), None),
            (
                "Mordor 2033".into(),
                "George R.R. Martin".into(),
                Some("Dmitrij Gluchovskij".into())
            ),
            ("The Hobbit".into(), "J.R.R. Tolkien".into(), None),
        ])
    );
}
