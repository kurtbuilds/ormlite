use chrono::{DateTime, Utc};
use ormlite::model::*;
use ormlite::types::Json;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(sqlx::Type)]
#[repr(i8)]
enum JobType {
    All = 1,
    Any = 2,
}

#[derive(Serialize, Deserialize)]
struct JobData {
    count: i32,
    value: String,
    timestamp: DateTime<Utc>,
}


#[derive(Model)]
struct Job {
    id: i32,
    typ: JobType,
    name: String,
    data: Json<JobData>,
    #[ormlite(skip)]
    skipped: Option<Uuid>,
}

#[derive(IntoArguments)]
struct ApiJob {
    id: i32,
    typ: JobType,
    name: String,
    #[ormlite(experimental_encode_as_json)]
    data: JobData,
}

#[tokio::main]
async fn main() {
    assert_eq!(Job::table_columns(), vec![
        "id".to_string(),
        "typ".to_string(),
        "name".to_string(),
        "data".to_string(),
    ]);
}