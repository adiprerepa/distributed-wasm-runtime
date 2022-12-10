
pub mod filters {
    use warp::Filter;
    use distributed_wasm_runtime::modules::WasmPayload;
    use crate::handlers::handlers;

    pub fn workers() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        worker_accept_job()
    }

    pub fn worker_accept_job() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("accept_job")
            .and(warp::post())
            .and(warp::body::json())
            .and_then(handlers::handle_job_request)
    }
}