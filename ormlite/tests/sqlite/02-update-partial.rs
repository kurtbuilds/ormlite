use ormlite::model::*;
use ormlite::Connection;
use sqlmo::ToSql;
use uuid::Uuid;

#[derive(Model)]
pub struct Person {
    id: Uuid,
    name: String,
    age: u8,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = ormlite::sqlite::SqliteConnection::connect(":memory:").await.unwrap();
    let migration = ormlite::__private::migrate_self(&[file!()]);
    for s in migration.statements {
        let sql = s.to_sql(sqlmo::Dialect::Sqlite);
        ormlite::query(&sql)
            .execute(&mut db)
            .await?;
    }

    let p = Person {
        id: Uuid::new_v4(),
        name: "John".to_string(),
        age: 99,
    }.insert(&mut db).await?;

    let p = p.update_partial()
        .age(100)
        .update(&mut db)
        .await?;

    assert_eq!(p.age, 100);
    Ok(())
}
