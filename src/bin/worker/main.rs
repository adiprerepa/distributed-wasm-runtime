mod filters;
mod handlers;

use std::env;
use distributed_wasm_runtime::modules::*;
use crate::filters::filters::workers;
use warp::Filter;

#[tokio::main]
async fn main() {
    // let bind_port = match env::args().nth(0) {
    //     Some(port) => port.parse::<i32>().unwrap(),
    //     None => 3031,
    // };

    let api = workers();
    let workers = api.with(warp::log("dwasm-worker"));

    warp::serve(workers)
        .run(([127, 0, 0, 1], 3031)).await;
}