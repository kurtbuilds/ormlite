#![allow(unused)]
#![cfg(not(test))]
use ormlite_macro::Model;

#[derive(Model)]
pub struct Person {
    id: u32,
    name: String,
    age: u8,
}

#[derive(Model)]
#[ormlite(table_name = "person", primary_key = "name")]
pub struct InsertPerson {
    name: String,
    age: u8,
}

fn main() {}
