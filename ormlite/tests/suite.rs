#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-update-partial.rs");
    t.pass("tests/02-many-to-one.rs");
    t.pass("tests/03-many-to-many.rs");
    t.pass("tests/04-many-to-one.rs");
}