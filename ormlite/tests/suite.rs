use std::env;
use std::path::Path;
use trybuild::TestCases;

static DIR: &str = std::env!("CARGO_MANIFEST_DIR");

fn set_path_and_run(path: &str, t: &TestCases) {
    let p = Path::new(DIR).join(path);
    env::set_var("MODEL_FOLDERS", p.display().to_string());
    t.pass(path);
}

#[test]
fn test_suite() {

    let t = TestCases::new();
    t.pass("tests/01-update-partial.rs");
    set_path_and_run("tests/02-many-to-one.rs", &t);
    t.pass("tests/table-meta.rs");
    // t.pass("tests/03-many-to-many.rs");
    // t.pass("tests/04-one-to-many.rs");
}