pub mod filters {
    use warp::Filter;
    use distributed_wasm_runtime::modules::{Db, Job, JobOptions, JobUpdate, Worker, WorkerIndex};
    use crate::handlers;
    use super::handlers;

    // combined filters
    pub fn jobs(db: Db, worker_index: WorkerIndex) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        job_create(db.clone())
            .or(job_status(db.clone()))
            .or(job_update(db))
            .or(register_worker(worker_index))
    }

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
