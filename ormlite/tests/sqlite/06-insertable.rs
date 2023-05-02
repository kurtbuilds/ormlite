use ormlite::Model;
use ormlite::model::Insertable;
use sqlmo::ToSql;

use ormlite::Connection;
#[path = "../setup.rs"]
mod setup;

#[derive(Model)]
#[ormlite(insertable = InsertUser)]
pub struct User {
    id: i32,
    name: String,
    #[ormlite(default)]
    secret: Option<String>,
    #[ormlite(default_value = "5")]
    number: i32,
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

    let user = InsertUser {
        name: "Champ".to_string(),
    }.insert(&mut db)
        .await
        .unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.secret, None);
    assert_eq!(user.number, 5);
}