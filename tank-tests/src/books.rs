// use std::{collections::HashSet, sync::LazyLock};
// use tank::{
//     Connection, DataSet, Entity, Passive, RowLabeled, Value, expr, join,
//     stream::{StreamExt, TryStreamExt},
// };
// use tokio::sync::Mutex;
// use uuid::Uuid;

// static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

// #[derive(Entity, Debug, Clone)]
// #[tank(schema = "testing", name = "authors")]
// pub struct Author {
//     #[tank(primary_key, name = "author_id")]
//     pub id: Passive<Uuid>,
//     pub name: String,
//     pub country: String,
// }

// #[derive(Entity, Debug, Clone)]
// #[tank(schema = "testing", name = "books")]
// pub struct Book {
//     #[tank(primary_key)]
//     pub isbn: [u8; 13],
//     pub title: String,
//     /// Main author
//     pub author: Uuid,
//     pub co_author: Option<Uuid>,
//     pub year: i32,
// }

// pub async fn books<C: Connection>(connection: &mut C) {
//     let _lock = MUTEX.lock().await;

//     // Setup
//     Book::drop_table(connection, true, false)
//         .await
//         .expect("Failed to drop Book table");
//     Author::drop_table(connection, true, false)
//         .await
//         .expect("Failed to drop Author table");
//     Author::create_table(connection, false, true)
//         .await
//         .expect("Failed to create Author table");
//     Book::create_table(connection, false, true)
//         .await
//         .expect("Failed to create Book table");

//     // Author objects
//     let authors = vec![
//         Author {
//             id: Uuid::parse_str("f938f818-0a40-4ce3-8fbc-259ac252a1b5")
//                 .unwrap()
//                 .into(),
//             name: "J.R.R. Tolkien".into(),
//             country: "UK".into(),
//         },
//         Author {
//             id: Uuid::parse_str("a73bc06a-ff89-44b9-a62f-416ebe976285")
//                 .unwrap()
//                 .into(),
//             name: "George R.R. Martin".into(),
//             country: "USA".into(),
//         },
//         Author {
//             id: Uuid::parse_str("6b2f56a1-316d-42b9-a8ba-baca42c5416c")
//                 .unwrap()
//                 .into(),
//             name: "Dmitrij Gluchovskij".into(),
//             country: "Russia".into(),
//         },
//         Author {
//             id: Uuid::parse_str("d3d3d3d3-d3d3-d3d3-d3d3-d3d3d3d3d3d3")
//                 .unwrap()
//                 .into(),
//             name: "Linus Torvalds".into(),
//             country: "Finland".into(),
//         },
//     ];
//     let tolkien_id = authors[0].id.unwrap();
//     let martin_id = authors[1].id.unwrap();
//     let gluchovskij_id = authors[2].id.unwrap();

//     // Book objects
//     let books = vec![
//         Book {
//             isbn: [9, 7, 8, 0, 0, 0, 7, 4, 4, 0, 8, 3, 2],
//             title: "The Hobbit".into(),
//             author: tolkien_id,
//             co_author: None,
//             year: 1937,
//         },
//         Book {
//             isbn: [9, 7, 8, 0, 0, 0, 2, 2, 4, 5, 8, 4, 5],
//             title: "A Game of Thrones (A Song of Ice and Fire book 1) ".into(),
//             author: martin_id,
//             co_author: None,
//             year: 1996,
//         },
//         Book {
//             isbn: [9, 7, 8, 5, 1, 7, 0, 5, 9, 6, 7, 8, 2],
//             title: "Metro 2033".into(),
//             author: gluchovskij_id,
//             co_author: None,
//             year: 2002,
//         },
//         Book {
//             isbn: [9, 7, 8, 0, 0, 2, 3, 4, 5, 6, 7, 8, 9],
//             title: "Mordor 2033".into(),
//             author: tolkien_id,
//             co_author: gluchovskij_id.into(),
//             year: 2003,
//         },
//     ];

//     // Insert
//     let result = Author::insert_many(connection, authors.iter())
//         .await
//         .expect("Failed to insert authors");
//     assert_eq!(result.rows_affected, 4);
//     let result = Book::insert_many(connection, books.iter())
//         .await
//         .expect("Failed to insert books");
//     assert_eq!(result.rows_affected, 4);

//     // Get books before 2000
//     let join = join!(Book B JOIN Author A ON B.author_id == A.author_id);
//     let result = join
//         .select(
//             &[expr!(B.title), expr!(A.name)],
//             connection,
//             &expr!(B.publication_year < 2000),
//             None,
//         )
//         .try_collect::<Vec<RowLabeled>>()
//         .await
//         .expect("Failed to query books and authors joined")
//         .into_iter()
//         .map(|row| {
//             (
//                 match row.values()[0] {
//                     Value::Varchar(Some(v)) => v,
//                     _ => panic!("Expected first value to be non null varchar"),
//                 },
//                 match row.values()[1] {
//                     Value::Varchar(Some(v)) => v,
//                     _ => panic!("Expected second value to be non null varchar"),
//                 },
//             )
//         })
//         .collect::<HashSet<_>>();
//     assert_eq!(
//         result,
//         HashSet::from_iter([
//             ("The Hobbit".into(), "J.R.R. Tolkien".into()),
//             ("A Game of Thrones".into(), "George R.R. Martin".into()),
//         ])
//     );

//     let join = join!(Book B LEFT JOIN Author CO_A ON B.co_author_id == CO_A.author_id);
//     let results = join
//         .select(
//             &[expr!(B.title), expr!(CO_A.name as co_author_name)],
//             connection,
//             &true, // No specific filter
//             None,
//         )
//         .try_collect::<Vec<RowLabeled>>()
//         .await
//         .unwrap();

//     assert_eq!(results.len(), 3, "Should return all 3 books");
//     let mut with_co_author = 0;
//     let mut without_co_author = 0;
//     for row in results {
//         let title = row.get::<String, _>("title").unwrap();
//         let co_author_name = row.get::<Option<String>, _>("co_author_name").unwrap();
//         if title == "A Clash of Kings" {
//             assert_eq!(co_author_name, Some("Andrzej Sapkowski".into()));
//             with_co_author += 1;
//         } else {
//             assert!(co_author_name.is_none());
//             without_co_author += 1;
//         }
//     }
//     assert_eq!(with_co_author, 1);
//     assert_eq!(without_co_author, 2);

//     // Test Case 3: Double JOIN to get a book, its author, AND its co-author.
//     println!("--- Running Test Case 3: Double JOIN ---");
//     let join = join! {
//         Book B
//         JOIN Author A ON B.author_id == A.author_id
//         JOIN Author CO_A ON B.co_author_id == CO_A.author_id
//     };
//     let results = join
//         .select::<_, _, _, _>(
//             &["B.title", "A.name as author", "CO_A.name as co_author"],
//             connection,
//             &expr!(B.title == "A Clash of Kings"),
//             None,
//         )
//         .try_collect::<Vec<RowLabeled>>()
//         .await
//         .unwrap();

//     assert_eq!(
//         results.len(),
//         1,
//         "Expected to find only one co-authored book"
//     );
//     let row = &results[0];
//     assert_eq!(row.get::<String, _>("title").unwrap(), "A Clash of Kings");
//     assert_eq!(
//         row.get::<String, _>("author").unwrap(),
//         "George R.R. Martin"
//     );
//     assert_eq!(
//         row.get::<String, _>("co_author").unwrap(),
//         "Andrzej Sapkowski"
//     );

//     // Test Case 4: RIGHT JOIN to find all authors, and any books they may have.
//     // This should include C.S. Lewis, who has no books in our DB.
//     println!("--- Running Test Case 4: RIGHT JOIN ---");
//     let join = join!(Book B RIGHT JOIN Author A ON B.author_id == A.author_id);
//     let results = join
//         .select::<_, _, _, _>(&["A.name", "B.title"], connection, &true, None)
//         .try_collect::<Vec<RowLabeled>>()
//         .await
//         .unwrap();

//     assert_eq!(
//         results.len(),
//         4,
//         "Should return a row for each author, even those without books"
//     );
//     let mut found_lewis = false;
//     for row in results {
//         let author_name = row.get::<String, _>("name").unwrap();
//         let book_title = row.get::<Option<String>, _>("title").unwrap();
//         if author_name == "C.S. Lewis" {
//             assert!(
//                 book_title.is_none(),
//                 "C.S. Lewis should not have a book title associated"
//             );
//             found_lewis = true;
//         }
//     }
//     assert!(
//         found_lewis,
//         "Did not find C.S. Lewis in the RIGHT JOIN result"
//     );
// }
