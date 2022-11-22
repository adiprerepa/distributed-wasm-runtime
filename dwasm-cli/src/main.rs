use serde::{Deserialize, Serialize};
use crate::models::{CreateJobResponse, Job};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let job = models::Job {
        rust_src: String::from("hello"),
        cpus: 1,
        memory_mb: 2,
        job_name: String::from("job1")
    };
    let new_job: CreateJobResponse = reqwest::Client::new()
        .post("http://127.0.0.1:3030/new_job")
        .json(&job)
        .send()
        .await?
        .json()
        .await?;
    println!("{:?}", new_job);
    Ok(())
}

mod models {
    use std::collections::HashMap;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use std::time::SystemTime;
    use tokio::sync::Mutex;

    pub type Db = Arc<Mutex<HashMap<i32, JobModel>>>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Job {
        pub rust_src: String,
        pub cpus: i32,
        pub memory_mb: i32,
        pub job_name: String,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct JobModel {
        pub job: Job,
        pub finished: bool,
        pub finished_at: u64,
        pub started_at: u64,
        pub job_id: i32,
        pub exec_output: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct JobOptions {
        pub id: Option<i32>,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct CreateJobResponse {
        pub id: i32,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct ErrorResponse {
        message: String,
    }

    pub fn blank_db() -> Db {
        return Arc::new(Mutex::new(HashMap::new()));
    }
}