
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
            .and(json_body_wasm_payload())
            .and_then(handlers::handle_job_request)
    }

    pub fn json_body_wasm_payload() -> impl Filter<Extract = (WasmPayload,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024*1024).and(warp::body::json())
    }
}