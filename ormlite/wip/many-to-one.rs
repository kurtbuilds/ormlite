// use ormlite::model::*;
// use ormlite::Connection;
// use uuid::Uuid;
//
// #[derive(Model, Clone)]
// pub struct Organization {
//     id: i32,
//     name: String,
//     #[ormlite(skip)]
//     extension: Option<String>,
// }
//
// #[derive(Model)]
// #[ormlite(insertable = InsertUser)]
// pub struct User {
//     id: i32,
//     name: String,
//     #[ormlite(default)]
//     secret: Option<String>,
//     #[ormlite(default_value = "5")]
//     number: i32,
//     #[ormlite(column = "type")]
//     typ: i32,
//     #[ormlite(join_column = "org_id")]
//     organization: Join<Organization>,
// }
//
// pub static CREATE_PERSON_SQL: &str = r#"
// CREATE TABLE user (
// id INTEGER PRIMARY KEY
// , name TEXT NOT NULL
// , secret TEXT
// , number INTEGER
// , type INTEGER
// , org_id INTEGER
// )
// "#;
//
// pub static CREATE_ORG_SQL: &str = r#"
// CREATE TABLE organization (
// id INTEGER PRIMARY KEY
// , name TEXT
// )
// "#;
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     env_logger::init();
//     let mut db = ormlite::sqlite::SqliteConnection::connect(":memory:").await.unwrap();
//     ormlite::query(CREATE_PERSON_SQL)
//         .execute(&mut db)
//         .await?;
//     ormlite::query(CREATE_ORG_SQL)
//         .execute(&mut db)
//         .await?;
//
//     let org = Organization {
//         id: 12321,
//         name: "my org".to_string(),
//         extension: None,
//     };
//
//     let champ = InsertUser {
//         name: "Champ".to_string(),
//         organization: Join::new(org.clone()),
//         typ: 12,
//     }.insert(&mut db)
//         .await
//         .unwrap();
//
//     assert_eq!(champ.id, 1);
//     assert_eq!(champ.secret, None);
//     assert_eq!(champ.number, 5);
//     assert_eq!(champ.organization.id, 12321);
//     assert_eq!(champ.organization.name, "my org");
//
//     let millie = InsertUser {
//         name: "Millie".to_string(),
//         organization: Join::new(org),
//         typ: 3,
//     }.insert(&mut db)
//         .await
//         .unwrap();
//     assert_eq!(millie.id, 2);
//     assert_eq!(millie.secret, None);
//     assert_eq!(millie.number, 5);
//     assert_eq!(millie.organization.id, 12321);
//     assert_eq!(millie.organization.name, "my org");
//
//     let enoki = InsertUser {
//         name: "Enoki".to_string(),
//         organization: Join::new_with_id(12321),
//         typ: 6,
//     }.insert(&mut db)
//         .await
//         .unwrap();
//     assert_eq!(enoki.id, 3);
//     assert_eq!(enoki.secret, None);
//     assert_eq!(enoki.number, 5);
//     assert_eq!(enoki.organization.id, 12321);
//     assert_eq!(enoki.organization.loaded(), false);
//
//     Ok(())
// }

fn main() {

}