use ormlite::Model;
use ormlite::TableMeta;
use sqlmo::ToSql;

use ormlite::Connection;
#[path = "../setup.rs"]
mod setup;

#[derive(Model)]
pub struct User {
    id: i32,
    #[ormlite(column = "type")]
    typ: String,
}

#[tokio::main]
async fn main() {
    let mut db = ormlite::sqlite::SqliteConnection::connect(":memory:")
        .await
        .unwrap();
    let migration = setup::migrate_self(&[file!()]);
    for s in migration.statements {
        let sql = s.to_sql(sqlmo::Dialect::Sqlite);
        ormlite::query(&sql)
            .execute(&mut db)
            .await
            .unwrap();
    }

    User {
        id: 1,
        typ: "admin".to_string(),
    }.insert(&mut db)
        .await
        .unwrap();

    let users = User::select()
        .fetch_all(&mut db)
        .await
        .unwrap();

    assert_eq!(User::table_name(), "user");
    assert_eq!(User::table_columns(), &["id", "type"]);

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].typ, "admin");
}