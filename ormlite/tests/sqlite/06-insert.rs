use ormlite::model::{Insert, Join, JoinMeta, Model};
use sqlmo::ToSql;
use serde::{Serialize, Deserialize};
use serde_json::json;

use ormlite::Connection;
#[path = "../setup.rs"]
mod setup;

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
pub struct Organization {
    id: i32,
    name: String,
}

#[derive(Model)]
#[ormlite(insert = "InsertUser", extra_derives(Serialize, Deserialize))]
// Note the previous syntax, #[ormlite(insertable = InsertUser)] still works, but the new syntax is preferred.
pub struct User {
    id: i32,
    name: String,
    #[ormlite(default)]
    secret: Option<String>,
    #[ormlite(default_value = "5")]
    number: i32,
    #[ormlite(column = "type")]
    ty: i32,
    #[ormlite(join_column = "org_id")]
    organization: Join<Organization>,
}

#[derive(Insert)]
#[ormlite(returns = "User")]
pub struct InsertUser2 {
    name: String,
    number: i32,
    #[ormlite(column = "type")]
    ty: i32,
    org_id: i32,
}

#[tokio::main]
async fn main() {
    let mut db = ormlite::sqlite::SqliteConnection::connect(":memory:").await.unwrap();
    let migration = setup::migrate_self(&[file!()]);
    for s in migration.statements {
        let sql = s.to_sql(sqlmo::Dialect::Sqlite);
        ormlite::query(&sql).execute(&mut db).await.unwrap();
    }

    let org = Organization {
        id: 12321,
        name: "my org".to_string(),
    };

    let champ = InsertUser {
        name: "Champ".to_string(),
        organization: Join::new(org.clone()),
        ty: 12,
    }
    .insert(&mut db)
    .await
    .unwrap();

    assert_eq!(champ.id, 1);
    assert_eq!(champ.secret, None);
    assert_eq!(champ.number, 5);
    assert_eq!(champ.organization.id, 12321);
    assert_eq!(champ.organization.name, "my org");

    let champ_copy = InsertUser {
        name: "Champ".to_string(),
        organization: Join::new(org.clone()),
        ty: 12,
    };
    let champ_json = json!(champ_copy).to_string();

    assert_eq!(champ_json, r#"{"name":"Champ","organization":{"id":12321,"name":"my org"},"ty":12}"#);

    let champ_deserializing = serde_json::from_str::<InsertUser>(r#"{"name":"Champ","organization":{"id":12321,"name":"my org"},"ty":12}"#);

    let Ok(champ_deserialized) = champ_deserializing else {
        panic!("Deserialize failing");
    };

    assert_eq!(champ_deserialized.name, champ_copy.name);
    assert_eq!(champ_deserialized.organization.name, champ_copy.organization.name);

    let millie = InsertUser {
        name: "Millie".to_string(),
        organization: Join::new(org),
        ty: 3,
    }
    .insert(&mut db)
    .await
    .unwrap();
    assert_eq!(millie.id, 2);
    assert_eq!(millie.secret, None);
    assert_eq!(millie.number, 5);
    assert_eq!(millie.organization.id, 12321);
    assert_eq!(millie.organization.name, "my org");

    let enoki = InsertUser {
        name: "Enoki".to_string(),
        organization: Join::new_with_id(12321),
        ty: 6,
    }
    .insert(&mut db)
    .await
    .unwrap();
    assert_eq!(enoki.id, 3);
    assert_eq!(enoki.secret, None);
    assert_eq!(enoki.number, 5);
    assert_eq!(enoki.organization.id, 12321);
    assert_eq!(enoki.organization.loaded(), false);

    let user = InsertUser2 {
        name: "Kloud".to_string(),
        number: 12,
        ty: 8,
        org_id: 12321,
    }
    .insert(&mut db)
    .await
    .unwrap();
    assert_eq!(user.id, 4);
    assert_eq!(user.name, "Kloud");
    let user = User::fetch_one(4, &mut db).await.unwrap();
    assert_eq!(user.id, 4);
    assert_eq!(user.name, "Kloud");
    assert_eq!(user.ty, 8);
    assert_eq!(user.organization.id, 12321);
}
