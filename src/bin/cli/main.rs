use serde::{Deserialize, Serialize};
use distributed_wasm_runtime::modules::CreateJobResponse;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let job = models::Job {
        rust_src: String::from("hello"),
        cpus: 1,
        memory_mb: 2,
        job_name: String::from("job1")
    };
    let new_job: CreateJobResponse = reqwest::Client::new()
        .post("http://127.0.0.1:3030/new_job")
        .json(&job)
        .send()
        .await?
        .json()
        .await?;
    println!("{:?}", new_job);
    Ok(())
}
