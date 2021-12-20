

#[derive(Clone, Debug, PartialEq, sea_orm::DeriveEntityModel)]
#[sea_orm(table_name = "person")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: u32,
    name: String,
    age: u8,
}

impl sea_orm::ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, sea_orm::EnumIter, sea_orm::DeriveRelation)]
pub enum Relation {}