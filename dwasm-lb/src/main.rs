use warp::Filter;
use crate::models::{blank_db, blank_worker_index};
use rand::Rng;


#[tokio::main]
async fn main() {
    let db = blank_db();
    let worker_index = blank_worker_index();

    let api = filters::jobs(db, worker_index);

    let jobs = api.with(warp::log("dwasm-lb"));
    warp::serve(jobs)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

mod filters {
    use warp::Filter;
    use crate::models::{Db, Job, JobOptions, JobUpdate, Worker, WorkerIndex};
    use super::handlers;

    // combined filters
    pub fn jobs(db: Db, worker_index: WorkerIndex) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        job_create(db.clone())
            .or(job_status(db.clone()))
            .or(job_update(db))
            .or()
    }

    // jobs

    // POST /new_job
    pub fn job_create(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("new_job")
            .and(warp::post())
            .and(json_body_job())
            .and(with_db(db))
            .and_then(handlers::create_job)
    }

    // GET /job_status
    pub fn job_status(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("job_status")
            .and(warp::get())
            .and(warp::query::<JobOptions>())
            .and(with_db(db))
            .and_then(handlers::status_job)
    }

    // POST /job_update/<id>
    pub fn job_update(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("job_update" / i32)
            .and(warp::post())
            .and(json_body_job_update())
            .and(with_db(db))
            .and_then(handlers::job_update)
    }

    // workers

    // POST /register_worker
    pub fn register_worker(worker_index: WorkerIndex) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("register_worker")
            .and(warp::post())
            .and(json_body_register_worker())
            .and(with_worker_index(worker_index))
            .and(warp::addr::remote())
            .and_then(handlers::register_worker)
    }



    fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn with_worker_index(worker_index: WorkerIndex) -> impl Filter<Extract = (WorkerIndex,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || worker_index.clone())
    }

    fn json_body_job() -> impl Filter<Extract = (Job,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body
        // (and to reject huge payloads)...
        warp::body::content_length_limit(1024 * 64).and(warp::body::json())
    }

    fn json_body_job_update() -> impl Filter<Extract = (JobUpdate,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body
        // (and to reject huge payloads)...
        warp::body::content_length_limit(1024 * 64).and(warp::body::json())
    }

    fn json_body_register_worker() -> impl Filter<Extract = (Worker,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024*64).and(warp::body::json())
    }
}

mod handlers {
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::SystemTime;
    use super::models::Db;
    use super::models::{Job, JobOptions};
    use rand::{prelude, Rng};
    use rand::rngs::ThreadRng;
    use tokio::sync::MutexGuard;
    use warp::http::StatusCode;
    use warp::{Filter, http::Response};
    use crate::models::{CreateJobResponse, JobModel, JobUpdate, Worker, WorkerIndex};

    pub async fn create_job(job: Job, db: Db) -> Result<impl warp::Reply, Infallible> {
        println!("create job: {:?}", job);
        let mut map: MutexGuard<HashMap<i32, JobModel>> = db.lock().await;
        let mut rng: ThreadRng = rand::thread_rng();
        let mut id: i32 = 0;
        let base: i32 = 10;
        loop {
            id = rng.gen_range(0..base.pow(8));
            if !map.contains_key(&id) {
                break;
            }
        }
        let started_at = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        map.insert(id, JobModel{
            job: job.clone(),
            finished: false,
            finished_at: 0,
            started_at: started_at.as_secs(),
            job_id: id,
            exec_output: String::from(""),
        });
        let json = warp::reply::json(&CreateJobResponse{ id });
        Ok(warp::reply::with_status(json, StatusCode::OK))
    }

    pub async fn status_job(opts: JobOptions, db: Db) -> Result<impl warp::Reply, Infallible> {
        let map = db.lock().await;
        let id = match opts.id {
            Some(i) => i,
            None => -1,
        };
        if id < 0 {
            return Ok(warp::reply::with_status(warp::reply::json(&()), StatusCode::BAD_REQUEST));
        }
        if !map.contains_key(&id) {
            return Ok(warp::reply::with_status(warp::reply::json(&()), StatusCode::NOT_FOUND));
        }
        let job = map.get(&id).unwrap().clone();
        let json = warp::reply::json(&job);
        return Ok(warp::reply::with_status(json, StatusCode::OK));
    }

    pub async fn job_update(id: i32, update: JobUpdate, db: Db) -> Result<impl warp::Reply, Infallible> {
        println!("job update: {:?}", update);
        let mut map = db.lock().await;
        if !map.contains_key(&id) {
            return Ok(StatusCode::NOT_FOUND);
        }
        let mut model = map.get(&id).unwrap().clone();
        model.finished = true;
        model.exec_output = update.exec_output;
        model.finished_at = update.finished_at;
        println!("updated to: {:?}", model.clone());
        map.insert(id, model);
        return Ok(StatusCode::OK);
    }

    // todo check if the worker should really send it's own ip address over the connection (is it necessary?)
    pub async fn register_worker(worker: Worker, worker_index: WorkerIndex, addr: Option<SocketAddr>) -> Result<impl warp::Reply, Infallible> {
        println!("registering worker: {:?}", worker);
        let ip: IpAddr = match addr {
            Some(x) => x.ip().clone(),
            None => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        };
        let mut to_insert = worker.clone();
        to_insert.ip_addr = ip.into_string();
        let mut index = worker_index.lock().await;
        index.insert(ip, to_insert);
        Ok(StatusCode::OK)
    }
}

mod wasm {
    use std::ops::Add;
    use std::process::Command;

    // takes rust source path, returns compiled wasm path
    pub fn compile_wasm(rust_file: &str, job_id: &str) -> String {
        let wasm_file_name = String::from("/tmp/".to_owned().add(&job_id.to_owned()).add(".wasm"));
        // rustc <rust_file> -o <wasm_file_name> --target wasm32-wasi
        Command::new("rustc")
            .args([rust_file, "-o", &wasm_file_name.clone(), "--target", "wasm32-wasi"])
            .output().expect("");
        wasm_file_name
    }
}

mod workers {
    use std::net::IpAddr;
    use crate::models::{Job, Worker};

    pub fn match_worker(job: Job) -> IpAddr {
        // algorithm for matching job to worker
        // minimize(sum of abs(job.constraint - worker_n.constraint))
        // weight cpu/memory as 0.5
        todo!()
    }

    // do we return statuses from here?
    pub fn worker_request_job(job: Job, worker: Worker) -> None {

    }

}

mod models {
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
        rust_src: String,
        cpus: i32,
        memory_mb: i32,
        job_name: String,
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

    pub fn blank_db() -> Db {
        return Arc::new(Mutex::new(HashMap::new()));
    }

    pub fn blank_worker_index() -> WorkerIndex {
        return Arc::new(Mutex::new(HashMap::new()))
    }
}