use std::fs;
use serde::{Deserialize, Serialize};
use distributed_wasm_runtime::modules::{CreateJobResponse, Job, JobModel, WasmRunResult};
use clap::{Args, Parser, Subcommand};
use warp::hyper::body::HttpBody;

/*
Cli should have two commands;

run <src>.rs --cpu 3 --memory 1000 --name job1

status <id> outputs job statistics
 */

#[derive(Parser)]
struct DwasmCli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value_t="127.0.0.1:3030")]
    server_endpoint: String,
}

#[derive(Subcommand)]
enum Commands {
    Run(Run),
    Status(Status)
}

#[derive(Args)]
struct Run {
    source: Option<String>,

    #[arg(short, long, default_value_t=1)]
    cpus: i32,

    #[arg(short, long, default_value_t=1000)]
    memory_mb: i32,

    #[arg(short, long, default_value_t="job_xyz")]
    job_name: String,
}

#[derive(Args)]
struct Status {
    #[arg(short, long)]
    id: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let cli = DwasmCli::parse();
    let client = reqwest::Client::new();
    let server_url = "http://".into_string() + cli.server_endpoint.as_str();

    match &cli.command {
        Commands::Run(run) => {
            let rs_source = match fs::read_to_string(run.source?) {
                Ok(data) => data,
                Err(e) => {
                    println!("unable to read file: {:?}", e);
                    ""
                }
            };
            let job = Job {
                rust_src: rs_source,
                cpus: run.cpus,
                memory_mb: run.memory_mb,
                job_name: run.job_name.clone(),
            };
            let response = client.post(server_url + "/new_job")
                .json(&job)
                .send()
                .await?
                .json::<CreateJobResponse>()
                .await?;
            println!("job running, id: {:?}.", response.id);
        }
        Commands::Status(status) => {
            let id = status.id?;
            let response = client.get(format!("{server_url}/job_status?id={id}"))
                .send()
                .await?
                .json::<JobModel>()
                .await?;
            println!("job status:\n{:?}", response);
        }
    }

    Ok(())
}
