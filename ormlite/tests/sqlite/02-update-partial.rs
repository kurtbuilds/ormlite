#[path = "../setup.rs"]
mod setup;

use ormlite::model::*;
use ormlite::Connection;
use sql::ToSql;
use uuid::Uuid;

#[derive(Model)]
pub struct Person {
    id: Uuid,
    name: String,
    age: u8,
}


#[tokio::main]
async fn main() {
    let mut db = ormlite::sqlite::SqliteConnection::connect(":memory:")
        .await
        .unwrap();
    let migration = setup::migrate_self(&[file!()]);
    for s in migration.statements {
        let sql = s.to_sql(sql::Dialect::Sqlite);
        ormlite::query(&sql)
            .execute(&mut db)
            .await
            .unwrap();
    }

    let p = Person {
        id: Uuid::new_v4(),
        name: "John".to_string(),
        age: 99,
    }.insert(&mut db)
        .await
        .unwrap();

    let p = p.update_partial()
        .age(100)
        .update(&mut db)
        .await
        .unwrap();

    assert_eq!(p.age, 100);
}
