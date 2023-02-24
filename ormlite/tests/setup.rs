
pub fn set_path_and_run(path: &str, t: &trybuild::TestCases) {
    static DIR: &str = env!("CARGO_MANIFEST_DIR");
    let p = std::path::Path::new(DIR).join(path);
    std::env::set_var("MODEL_FOLDERS", p.display().to_string());
    t.pass(path);
}

pub fn migrate_self(files: &[&str]) -> sqlmo::Migration {
    use ormlite_core::schema::TryFromOrmlite;
    let paths = files.iter().map(std::path::Path::new).collect::<Vec<_>>();
    let options = ormlite_core::schema::Options { verbose: false };
    let schema: sqlmo::Schema = TryFromOrmlite::try_from_ormlite_project(&paths, &options).unwrap();
    let migration = sqlmo::Schema::default().migrate_to(schema, &sqlmo::MigrationOptions::default()).unwrap();
    migration
}