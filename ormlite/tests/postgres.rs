#![cfg(not(any(feature = "sqlite", feature = "mysql")))]
#[path = "./run.rs"]
mod run;

use run::*;

#[test]
fn test_postgres_complex() {
    set_path_and_run("tests/postgres/complex.rs");
}

#[test]
fn test_postgres_join() {
    set_path_and_run("tests/postgres/join.rs");
}
