use ormlite::model::*;
use ormlite::{query, Connection};
use sqlmo::{Dialect, ToSql};
use uuid::Uuid;

#[path = "../tests/setup.rs"]
mod setup;

#[derive(Model)]
#[ormlite(table = "users")]
struct User {
    id: Uuid,
    name: String,
    #[ormlite(column = "org_id")]
    org: Join<Organization>,

    #[ormlite(column = "subscription_id")]
    subscription: Join<Option<Subscription>>,

    #[ormlite(foreign_field = Photo::user_id)]
    photos: Join<Vec<Photo>>,
}

#[derive(Model)]
struct Organization {
    id: Uuid,
    name: String,
}

#[derive(Model)]
struct Subscription {
    id: Uuid,
    name: String,
}

#[derive(Model)]
struct Photo {
    id: Uuid,
    user_id: Uuid,
    name: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let url = std::env::var("DATABASE_URL").unwrap();
    let mut conn = ormlite::postgres::PgConnection::connect(&url).await.unwrap();
    query("drop table if exists users, organization, photo")
        .execute(&mut conn)
        .await
        .unwrap();
    let migration = setup::migrate_self(&[file!()]);
    for s in migration.statements {
        let sql = s.to_sql(Dialect::Postgres);
        query(&sql).execute(&mut conn).await.unwrap();
    }
}
