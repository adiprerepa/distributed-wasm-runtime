
pub mod handlers {
    use std::convert::Infallible;
    use distributed_wasm_runtime::modules::WasmPayload;
    use std::io::Write;
    use std::fs;
    use std::path::Path;
    use wasmtime::*;
    use wasmtime_wasi::sync::WasiCtxBuilder;

    pub async fn handle_job_request(payload: WasmPayload) -> Result<impl warp::Reply, Infallible> {
        // write payload to tmp file
        let path= Path::new(&String::from("/tmp/" + payload.job_name + "run.wasm"));
        if !path.exists() {
            fs::File::create(path)?;
        }
        fs::write(path, payload.payload)?;

        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()
            .build();
        let mut store = Store::new(&engine, wasi);

        let module = Module::new(&engine, &payload.payload)?;
        linker.module(&mut store, "", &module)?;
        linker
            .get_default(&mut store, "")?
            .typed::<(), ()>(&store)?
            .call(&mut store, ())?;
    }
}