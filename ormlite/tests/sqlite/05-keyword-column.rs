use ormlite::Model;
use ormlite::TableMeta;

#[derive(Model)]
pub struct User {
    id: i32,
    #[ormlite(column = "type")]
    typ: String,
}

fn main() {
    assert_eq!(User::table_name(), "person");
    assert_eq!(User::table_columns(), &["id", "type"]);
}