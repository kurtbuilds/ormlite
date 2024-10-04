/// This is a helper function to run migrations for tests.
/// Import it from within a buildable test (i.e. a test built by trybuild) like so:
/// ```
/// #[path = "setup.rs"]
/// mod setup;
///
///

#[allow(dead_code)]
pub fn migrate_self(files: &[&str]) -> sqlmo::Migration {
    use ormlite_core::schema::schema_from_ormlite_project;
    let paths = files.iter().map(std::path::Path::new).collect::<Vec<_>>();
    let cfg = ormlite_core::config::Config::default();
    let schema: sqlmo::Schema = schema_from_ormlite_project(&paths, &cfg).unwrap();
    let opt = sqlmo::MigrationOptions::default();
    let migration = sqlmo::Schema::default().migrate_to(schema, &opt).unwrap();
    migration
}
