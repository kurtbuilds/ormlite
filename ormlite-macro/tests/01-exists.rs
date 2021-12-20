use ormlite_macro::Model;

#[derive(Model)]
pub struct Person {
    id: u32,
    name: String,
    age: u8,
}

fn main() {}
