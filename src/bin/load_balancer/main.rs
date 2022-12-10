mod filters;
mod handlers;
mod wasm;

use distributed_wasm_runtime::modules::*;
use crate::filters::filters::jobs;
use warp::Filter;

#[tokio::main]
async fn main() {
    let db: Db = blank_db();
    let worker_index: WorkerIndex = blank_worker_index();
    let api = jobs(db, worker_index);
    let jobs = api.with(warp::log("dwasm-lb"));

    warp::serve(jobs)
        .run(([127, 0, 0, 1], 3030))
        .await;

    println!("hi");
}