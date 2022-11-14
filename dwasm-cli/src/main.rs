use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Job {
    rust_src: String,
    cpus: i32,
    memory_mb: i32,
    job_name: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let job = Job {
        rust_src: String::from("hello"),
        cpus: 1,
        memory_mb: 2,
        job_name: String::from("job1")
    };
    let new_job: Job = reqwest::Client::new()
        .post("https://jsonplaceholder.typicode.com/posts")
        .json(&job)
        .send()
        .await?
        .json()
        .await?;
    println!("{:?}", new_job);
    Ok(())
}
