/// This is a helper function to run migrations for tests.
/// Import it from within a buildable test (i.e. a test built by trybuild) like so:
/// ```
/// #[path = "setup.rs"]
/// mod setup;
///
#[allow(dead_code)]
pub fn migrate_self(files: &[&str]) -> sqlmo::Migration {
    use ormlite_core::schema::TryFromOrmlite;
    let paths = files.iter().map(std::path::Path::new).collect::<Vec<_>>();
    let schema: sqlmo::Schema = TryFromOrmlite::try_from_ormlite_project(&paths).unwrap();
    let migration = sqlmo::Schema::default().migrate_to(schema, &sqlmo::MigrationOptions::default()).unwrap();
    migration
}
