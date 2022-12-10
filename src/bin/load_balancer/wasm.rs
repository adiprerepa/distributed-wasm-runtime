
pub mod wasm {
    use std::fs;
    use std::ops::Add;
    use std::process::Command;

    pub fn write_rust_src(rust_src: &str, job_id: &str) -> String {
        let path = "/tmp/" + job_id + ".rs";
        fs::write(path, rust_src).expect("unable to write file");
        path
    }

    // takes rust source path, returns compiled wasm path
    pub fn compile_wasm(rust_file: &str, job_id: &str) -> String {
        let wasm_file_name = String::from("/tmp/".to_owned().add(&job_id.to_owned()).add(".wasm"));
        // rustc <rust_file> -o <wasm_file_name> --target wasm32-wasi
        Command::new("rustc")
            .args([rust_file, "-o", &wasm_file_name.clone(), "--target", "wasm32-wasi"])
            .output().expect("");
        wasm_file_name
    }
}
