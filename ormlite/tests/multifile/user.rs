use crate::organization::Organization;
use ormlite::model::*;
use ormlite::types::Uuid;

#[derive(Debug, Model)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub age: u8,
    #[ormlite(column = "organization_id")]
    pub organization: Join<Organization>,
}
