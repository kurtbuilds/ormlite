use diesel;

#[derive(diesel::Queryable)]
pub struct DieselPerson {
    pub id: u32,
    pub name: String,
    pub age: u8,
}