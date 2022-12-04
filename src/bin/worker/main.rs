mod filters;
mod handlers;

use distributed_wasm_runtime::modules::*;
use crate::filters::filters::workers;

#[tokio::main]
async fn main() {
    let api = workers();
    let workers = api.with(warp::log("dwasm-worker"));

    warp::serve(workers)
        .run(([127, 0, 0, 1], 3030))
}