use warp::Filter;
use crate::models::blank_db;
use rand::Rng;


#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello"/ String)
        .map(|name| format!("Hello, {}!", name));

    let db = blank_db();

    let api = filters::jobs(db);

    let jobs = api.with(warp::log("dwasm-lb"));
    warp::serve(jobs)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

mod filters {
    use warp::Filter;
    use crate::models::{Db, Job, JobOptions};
    use super::handlers;

    // combined filters
    pub fn jobs(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        job_create(db.clone())
            .or(job_status(db))
    }

    // POST /new_job
    pub fn job_create(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("new_job")
            .and(warp::post())
            .and(json_body())
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

    fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn json_body() -> impl Filter<Extract = (Job,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body
        // (and to reject huge payloads)...
        warp::body::content_length_limit(1024 * 64).and(warp::body::json())
    }
}

mod handlers {
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::time::SystemTime;
    use super::models::Db;
    use super::models::{Job, JobOptions};
    use rand::{prelude, Rng};
    use rand::rngs::ThreadRng;
    use tokio::sync::MutexGuard;
    use warp::http::StatusCode;
    use warp::{Filter, http::Response};
    use crate::models::{CreateJobResponse, JobModel};

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