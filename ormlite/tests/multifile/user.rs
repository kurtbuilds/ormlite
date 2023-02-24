use ormlite::types::Uuid;
use ormlite::model::*;
use crate::organization::Organization;

#[derive(Debug, Model)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub age: u8,
    pub organization_id: Uuid,
    #[ormlite(many_to_one_key = organization_id)]
    pub organization: Join<Organization>,
}