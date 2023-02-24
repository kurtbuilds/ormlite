#[path = "./setup.rs"]
mod setup;

use trybuild::TestCases;

#[test]
fn test_sqlite() {
    let t = TestCases::new();
    t.pass("tests/sqlite/01-table-meta.rs");
    t.pass("tests/sqlite/02-update-partial.rs");
    setup::set_path_and_run("tests/03-many-to-one-join.rs", &t);
    t.pass("tests/sqlite/04-allow-clone-primary-key.rs");

    // t.pass("tests/table-meta.rs");

    // t.pass("tests/allow-clone-primary-key.rs");

    // t.pass("tests/03-many-to-many.rs");
    // t.pass("tests/04-one-to-many.rs");
}