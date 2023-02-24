#[path = "../setup.rs"]
mod setup;

mod user;
mod organization;

pub use user::User;
pub use organization::Organization;
use uuid::Uuid;
use ormlite::model::*;
use ormlite::sqlite::SqliteConnection;
use ormlite::Connection;

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::connect(":memory:").await?;
    let migration = setup::migrate_self(&[
        concat!(env!("CARGO_MANIFEST_DIR"), "tests/multifile/organization.rs"),
        concat!(env!("CARGO_MANIFEST_DIR"), "tests/multifile/user.rs"),
    ]);

    let org_id = Uuid::new_v4();
    let org = Organization {
        id: org_id,
        name: "Acme".to_string(),
        is_active: true,
    };
    let user = User {
        id: Uuid::new_v4(),
        name: "John".to_string(),
        age: 99,
        organization_id: Uuid::default(),
        organization: Join::new(org),
    };
    let user = user.insert(&mut conn).await?;
    assert_eq!(user.organization_id, org_id);
    Ok(())
}