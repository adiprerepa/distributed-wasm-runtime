
pub mod handlers {
    use std::convert::Infallible;
    use distributed_wasm_runtime::modules::{WasmPayload, WasmRunResult};
    use std::io::Write;
    use std::fs;
    use std::path::Path;
    use std::str::from_utf8;
    use reqwest::StatusCode;
    use wasi_common::pipe::WritePipe;
    use wasmtime::*;
    use wasmtime_wasi::sync::WasiCtxBuilder;

    pub async fn handle_job_request(payload: WasmPayload) -> Result<impl warp::Reply, Infallible> {
        // Define the WASI functions globally on the `Config`.
        println!("running {:?}", payload.job_name);
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        let out = WritePipe::new_in_memory();
        // Create a WASI context and put it in a Store; all instances in the store
        // share this context. `WasiCtxBuilder` provides a number of ways to
        // configure what the target program will have access to.
        let mut wasi = WasiCtxBuilder::new()
            .stdout(Box::new(out.clone()))
            .inherit_args()?
            .build();
        let mut store = Store::new(&engine, wasi);

        // Instantiate our module with the imports we've created, and run it.
        let module = Module::from_binary(&engine, &&payload.payload)?;
        linker.module(&mut store, "", &module)?;
        linker
            .get_default(&mut store, "")?
            .typed::<(), (), _>(&store)?
            .call(&mut store, ())?;
        drop(store);
        let res = from_utf8(out.try_into_inner().expect("shit").into_inner()?)?;
        println!("job result: {:?}", res);

        let wasm_result = WasmRunResult {output: String::from(res)};
        let json = warp::reply::json(&wasm_result);
        return Ok(warp::reply::with_status(json, StatusCode::OK));
    }
}