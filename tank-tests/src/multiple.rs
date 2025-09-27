use std::sync::LazyLock;
use tank::AsValue;
use tank::{Driver, Entity, Executor, QueryResult, RowLabeled, SqlWriter, stream::TryStreamExt};
use tokio::sync::Mutex;

#[derive(Entity)]
struct One {
    a1: u32,
    string: String,
    c1: u64,
}
#[derive(Entity)]
struct Two {
    a2: u32,
    string: String,
}
#[derive(Entity)]
struct Three {
    string: String,
}
static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub async fn multiple<E: Executor>(executor: &mut E) {
    let _lock = MUTEX.lock().await;

    let mut sql = String::new();
    sql.push_str("    \n\n  \n \n\t\t\n   \n    ");
    executor
        .driver()
        .sql_writer()
        .write_drop_table::<One>(&mut sql, true);
    sql.push_str("\t\t");
    executor
        .driver()
        .sql_writer()
        .write_drop_table::<Two>(&mut sql, true);
    executor
        .driver()
        .sql_writer()
        .write_drop_table::<Three>(&mut sql, true);
    executor
        .driver()
        .sql_writer()
        .write_create_table::<One>(&mut sql, true);
    sql.push('\n');
    executor
        .driver()
        .sql_writer()
        .write_create_table::<Two>(&mut sql, true);
    executor
        .driver()
        .sql_writer()
        .write_create_table::<Three>(&mut sql, true);
    sql.push_str(" ");
    executor.driver().sql_writer().write_insert(
        &mut sql,
        [
            &Two {
                a2: 21,
                string: "Two-1".into(),
            },
            &Two {
                a2: 22,
                string: "Two-2".into(),
            },
            &Two {
                a2: 23,
                string: "Two-3".into(),
            },
        ],
        false,
    );
    sql.push_str("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
    executor.driver().sql_writer().write_insert(
        &mut sql,
        [
            &Three {
                string: "Three-1".into(),
            },
            &Three {
                string: "Three-2".into(),
            },
        ],
        false,
    );
    executor.driver().sql_writer().write_select(
        &mut sql,
        [Three::string],
        Three::table_ref(),
        &true,
        None,
    );
    executor.driver().sql_writer().write_insert(
        &mut sql,
        [&One {
            a1: 11,
            string: "One-1".into(),
            c1: 512,
        }],
        false,
    );
    executor.driver().sql_writer().write_select(
        &mut sql,
        [One::a1, One::string, One::c1],
        One::table_ref(),
        &true,
        None,
    );
    executor.driver().sql_writer().write_select(
        &mut sql,
        [Two::a2, Two::string],
        Two::table_ref(),
        &true,
        None,
    );
    sql.push_str("            \t    \t\t  \n \n \n \t    \n\n\n ");
    let mut result = executor
        .run(sql.into())
        .try_filter_map(|v| async {
            Ok(match v {
                QueryResult::RowLabeled(row) => Some(row),
                QueryResult::Affected(..) => None,
            })
        })
        .try_collect::<Vec<RowLabeled>>()
        .await
        .expect("Could not run the composite query");
    result.sort_by(|a, b| {
        let a = a
            .get_column("string")
            .map(|v| String::try_from_value(v.clone()))
            .expect("Does not have column \"string\"")
            .expect("The column called `string` is not a VARCHAR");
        let b = b
            .get_column("string")
            .map(|v| String::try_from_value(v.clone()))
            .expect("Does not have column \"string\"")
            .expect("The column called `string` is not a VARCHAR");
        a.cmp(&b)
    });
    assert_eq!(*result[0].labels, ["a1", "string", "c1"]);
}
