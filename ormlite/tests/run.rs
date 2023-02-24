/// This is a utils mod for the test suite.
/// Import it from a test like so:
/// ```
/// #[path = "run.rs"]
/// mod run;
use trybuild::TestCases;

const FOO: &str = env!("CARGO_MANIFEST_DIR");


pub fn set_path_and_run(path: &str) {
    let t = TestCases::new();
    let p = std::path::Path::new(&FOO).join(path);
    std::env::set_var("MODEL_FOLDERS", p.display().to_string());
    t.pass(path);
}

// Ues if we have models across a directory, and MODEL_FOLDERS needs to be set.
pub fn set_dir_and_run(dir: &str, subpath: &str) {
    let t = TestCases::new();
    let p = std::path::Path::new(&FOO).join(dir);
    std::env::set_var("MODEL_FOLDERS", p.display().to_string());
    t.pass(p.join(subpath).display().to_string());
}