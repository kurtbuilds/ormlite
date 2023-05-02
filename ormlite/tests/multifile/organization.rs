use ormlite::types::Uuid;
use ormlite::model::*;

#[derive(Debug, Model)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
}

impl JoinMeta for Organization {
    type IdType = Uuid;
    fn _id(&self) -> Self::IdType {
        self.id.clone()
    }
}