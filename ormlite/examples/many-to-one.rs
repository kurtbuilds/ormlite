use ormlite::model::*;
use ormlite::Connection;
use uuid::Uuid;

#[derive(Model, Debug)]
pub struct Person {
    id: Uuid,
    name: String,
    age: u8,
    #[ormlite(join_column = "org_id")]
    organization: Join<Organization>,
}

#[derive(Model, Clone, Debug)]
#[ormlite(table = "orgs")]
pub struct Organization {
    id: Uuid,
    name: String,
}

impl JoinMeta for Organization {
    type IdType = Uuid;

    fn _id(&self) -> Self::IdType {
        self.id.clone()
    }
}

pub static CREATE_PERSON_SQL: &str =
    "CREATE TABLE person (id text PRIMARY KEY, name TEXT, age INTEGER, org_id text)";

pub static CREATE_ORG_SQL: &str =
    "CREATE TABLE orgs (id text PRIMARY KEY, name TEXT)";

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
    let p1 = Person {
        id: Uuid::new_v4(),
        name: "John".to_string(),
        age: 102,
        organization: Join::new(org.clone()),
    }.insert(&mut db).await.unwrap();
    assert_eq!(p1.organization.id, org.id, "setting the org object should overwrite the org_id field on insert.");
    assert_eq!(p1.organization.loaded(), true);

    let org = Organization::select()
        .where_bind("id = ?", &org.id)
        .fetch_one(&mut db)
        .await.unwrap();
    assert_eq!(org.name, "my org", "org gets inserted even though we didn't manually insert it.");

    let p2 = Person {
        id: Uuid::new_v4(),
        name: "p2".to_string(),
        age: 98,
        organization: Join::new(org.clone()),
    }.insert(&mut db)
        .await
        .unwrap();
    assert_eq!(p2.organization.id, org.id, "we can do insertion with an existing join obj, and it will pass the error.");

    let orgs = Organization::select()
        .fetch_all(&mut db)
        .await
        .unwrap();
    assert_eq!(orgs.len(), 1, "exactly 1 orgs");

    let people = Person::select()
        .fetch_all(&mut db)
        .await
        .unwrap();
    assert_eq!(people.len(), 2, "exactly 2 people");

    let people = Person::select()
        .join(Person::organization())
        .fetch_all(&mut db)
        .await
        .unwrap();
    assert_eq!(people.len(), 2, "exactly 2 people");
    for person in &people {
        assert_eq!(person.organization.name, "my org", "we can join on the org");
    }
    println!("people: {:#?}", people);
    Ok(())
}