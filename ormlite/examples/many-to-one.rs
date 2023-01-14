use ormlite::model::*;
use ormlite::Connection;
use uuid::Uuid;

#[derive(Model)]
pub struct Person {
    id: Uuid,
    name: String,
    age: u8,
    org_id: Uuid,
    // #[ormlite(many_to_one_key = Person::org_id())]
    #[ormlite(many_to_one_key = org_id)]
    organization: Join<Organization>,
}

#[derive(Model, Clone)]
pub struct Organization {
    id: Uuid,
    name: String,
}


pub static CREATE_PERSON_SQL: &str =
    "CREATE TABLE person (id text PRIMARY KEY, name TEXT, age INTEGER, org_id text)";

pub static CREATE_ORG_SQL: &str =
    "CREATE TABLE organization (id text PRIMARY KEY, name TEXT)";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut db = ormlite::sqlite::SqliteConnection::connect(":memory:").await.unwrap();
    ormlite::query(CREATE_PERSON_SQL)
        .execute(&mut db)
        .await?;
    ormlite::query(CREATE_ORG_SQL)
        .execute(&mut db)
        .await?;

    let org = Organization {
        id: Uuid::new_v4(),
        name: "my org".to_string(),
    };
    let p = Person {
        id: Uuid::new_v4(),
        name: "John".to_string(),
        age: 99,
        org_id: Uuid::default(),
        organization: Join::new(org.clone()),
    }.insert(&mut db).await?;
    assert_eq!(p.org_id, org.id, "setting the org object should overwrite the org_id field on insert.");

    let org = Organization::select()
        .where_("id = ?")
        .bind(&org.id)
        .fetch_one(&mut db)
        .await?;
    assert_eq!(org.name, "my org", "org gets inserted even though we didn't manually insert it.");

    // let p = p.update_partial()
    //     .age(100)
    //     .update(&mut db)
    //     .await?;
    //
    // assert_eq!(p.age, 100);
    Ok(())
}