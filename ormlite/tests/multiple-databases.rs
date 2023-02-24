/// Note this is work in progress and not working yet...
#[path = "run.rs"]
mod run;

use trybuild::TestCases;

#[test]
fn test_multifile() {
    run::set_dir_and_run("tests/multiple_databases", "main.rs");
}