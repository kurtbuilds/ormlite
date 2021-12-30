use ormlite::{FromRow, Model};

#[derive(Model, FromRow)]
pub struct Person {
    id: u32,
    name: String,
    age: u8,
}

fn main() {}
