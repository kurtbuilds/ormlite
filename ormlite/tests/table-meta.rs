use ormlite::model::*;
use uuid::Uuid;

#[derive(TableMeta)]
pub struct Person {
    id: Uuid,
    name: String,
    age: u8,
}


fn main() {
    assert_eq!(Person::table_name(), "person");
    assert_eq!(Person::table_columns(), &["id", "name", "age"]);
    assert_eq!(Person::primary_key(), Some("id"));
}
