pub mod modules {
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use std::time::SystemTime;
    use tokio::sync::Mutex;

    pub type Db = Arc<Mutex<HashMap<i32, JobModel>>>;
    pub type WorkerIndex = Arc<Mutex<HashMap<IpAddr, Worker>>>;

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
    pub struct JobUpdate {
        pub job_id: i32,
        pub exec_output: String,
        pub finished_at: u64,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct CreateJobResponse {
        pub id: i32,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct ErrorResponse {
        message: String,
    }

    // Workers
    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct Worker {
        pub ip_addr: String,
        pub port: i32,
        pub num_cpu: i32,
        pub memory_capacity_mb: i32,
        pub is_busy: bool,
        pub offline: bool,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct WasmPayload {
        pub payload: Vec<u8>,
        pub job_name: String,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct WasmRunResult {
        pub output: String,
    }

    pub fn blank_db() -> Db {
        return Arc::new(Mutex::new(HashMap::new()));
    }

    pub fn blank_worker_index() -> WorkerIndex {
        return Arc::new(Mutex::new(HashMap::new()))
    }
}

fn main() {}