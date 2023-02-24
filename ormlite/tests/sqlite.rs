use trybuild::TestCases;

static DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn set_path_and_run(t: &TestCases, path: &str) {
    let p = std::path::Path::new(DIR).join(path);
    std::env::set_var("MODEL_FOLDERS", p.display().to_string());
    t.pass(path);
}

pub fn set_dir_and_run(t: &TestCases, dir: &str, subpath: &str) {
    let p = std::path::Path::new(DIR).join(dir);
    std::env::set_var("MODEL_FOLDERS", p.display().to_string());
    t.pass(p.join(subpath).display().to_string());
}

#[cfg(not(any(feature = "postgres", feature = "mysql")))]
mod test {
    use super::*;

    #[test]
    fn test_sqlite() {
        let t = TestCases::new();
        t.pass("tests/sqlite/01-table-meta.rs");
        t.pass("tests/sqlite/02-update-partial.rs");
        set_path_and_run(&t, "tests/sqlite/03-many-to-one-join.rs");
        t.pass("tests/sqlite/04-allow-clone-primary-key.rs");
        // t.pass("tests/03-many-to-many.rs");
        // t.pass("tests/04-one-to-many.rs");
    }

    // #[test]
    // fn test_multifile() {
    //     let t = TestCases::new();
    //     set_dir_and_run(&t, "tests/multifile", "main.rs");
    // }
}