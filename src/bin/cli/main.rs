use std::borrow::Borrow;
use std::fs;
use serde::{Deserialize, Serialize};
use distributed_wasm_runtime::modules::{CreateJobResponse, Job, JobModel, WasmRunResult};
use clap::{Args, Parser, Subcommand};
use rand::Error;
use substring::Substring;
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

    #[arg(short, long)]
    server_endpoint: Option<String>,
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

    #[arg(short, long)]
    job_name: Option<String>,
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
    let endpoint = cli.server_endpoint.unwrap_or("127.0.0.1:3030".to_string());
    let server_url = format!("http://{endpoint}");

    match &cli.command {
        Some(Commands::Run(run)) => {
            let rs_source_path = match run.source.clone() {
                Some(path) => path,
                None => {
                    println!("rust path not found, please re-run.");
                    // todo better error handling here, it'll just output and keep running
                    String::from("")
                }
            };

            let rs_source = match fs::read_to_string(rs_source_path.to_string()) {
                Ok(data) => data,
                Err(e) => {
                    println!("unable to read file: {:?}", e);
                    String::from("")
                }
            };
            let job_name = run.job_name.clone().unwrap_or(rs_source_path.substring(0, rs_source_path.len() - 2).to_string());
            let job = Job {
                rust_src: rs_source,
                cpus: run.cpus,
                memory_mb: run.memory_mb,
                job_name,
            };
            let response = client.post(format!("{server_url}/new_job"))
                .json(&job)
                .send()
                .await?
                .json::<CreateJobResponse>()
                .await?;
            println!("job running, id: {:?}.", response.id);
        }
        Some(Commands::Status(status)) => {
            let id = match status.id {
                Some(id) => id,
                None => {
                    println!("please include an id");
                    0
                }
            };
            let response = client.get(format!("{server_url}/job_status?id={id}"))
                .send()
                .await?;

            if response.status() == 404 {
                println!("job id {:?} doesn't exist", id);
            }
            let job_model = response
                .json::<JobModel>()
                .await?;
            println!("job status:\n{:?}", job_model);
        }
        None => {
            println!("default subcommand");
        }
    }

    Ok(())
}
