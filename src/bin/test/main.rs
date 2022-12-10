use std::borrow::{Borrow, BorrowMut};
use std::str::from_utf8;
use anyhow::Result;
use wasi_common::pipe::WritePipe;
use wasi_common::WasiCtx;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

fn main() -> Result<()> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    let out = WritePipe::new_in_memory();
    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let mut wasi = WasiCtxBuilder::new()
        .stdout(Box::new(out.clone()))
        // .inherit_stdout()
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, wasi);

    // Instantiate our module with the imports we've created, and run it.
    let module = Module::from_file(&engine, "target/main.wasm")?;
    linker.module(&mut store, "", &module)?;
    linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store)?
        .call(&mut store, ())?;

    drop(store);
    let res = out.try_into_inner().expect("shit").into_inner();

    println!("{:?}", from_utf8(&&res)?);
    Ok(())
}