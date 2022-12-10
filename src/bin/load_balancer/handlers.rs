
pub mod handlers {
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::fs::File;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::SystemTime;
    use std::io::Read;
    use std::sync::Mutex;
    use rand::Rng;
    use distributed_wasm_runtime::modules::{CreateJobResponse, Db, Job, JobModel, JobOptions, JobUpdate, WasmPayload, WasmRunResult, Worker, WorkerIndex};
    use rand::rngs::ThreadRng;
    use reqwest::{Error, Response};
    use tokio::sync::MutexGuard;
    use warp::http::StatusCode;
    use crate::wasm::wasm;

    pub async fn create_job(job: Job, db: Db, worker_index: WorkerIndex) -> Result<impl warp::Reply, Infallible> {
        println!("create job: {:?}", job);
        let mut map: MutexGuard<HashMap<i32, JobModel>> = db.lock().await;
        let worker_index = worker_index.lock().await;
        let mut id: i32 = next_key(&map);

        let started_at = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let json = warp::reply::json(&CreateJobResponse{ id });

        // compile wasm
        let rust_src_file = wasm::write_rust_src(job.rust_src.as_str(), job.job_name.as_str());
        let mut wasm_file = File::open(wasm::compile_wasm(rust_src_file.as_str(), job.job_name.as_str())).unwrap();

        // read wasm into Vec<u8>
        let mut payload: Vec<u8> = Vec::new();
        wasm_file.read_to_end(&mut payload).unwrap();

        println!("payload length: {:?}", payload.len());

        // call worker (sync)
        let worker_ip = match_worker(job.cpus, job.memory_mb, worker_index).to_string();
        let endpoint: String = format!("http://{worker_ip}:3031/accept_job");
        let worker_request = WasmPayload {
            payload,
            job_name: job.job_name.clone(),
        };

        let client = reqwest::Client::new();
        let request_result = client.post(endpoint)
            .json(&worker_request)
            .send().await;
        let response = match request_result {
            Ok(resp) => {
                println!("{:?}", resp.status());
                resp
            },
            Err(e) => {
                println!("got error: {:?}", e);
                return Ok(warp::reply::with_status(json, StatusCode::SERVICE_UNAVAILABLE))
            }
        };

        let wasm_payload_result = response.json::<WasmRunResult>()
            .await.unwrap();
        println!("got output: {:?}", wasm_payload_result);

        let ended_at = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

        map.insert(id, JobModel{
            job: job.clone(),
            finished: true,
            finished_at: ended_at.as_secs(),
            started_at: started_at.as_secs(),
            job_id: id,
            exec_output: wasm_payload_result.output,
        });

        Ok(warp::reply::with_status(json, StatusCode::OK))
    }

    pub async fn status_job(opts: JobOptions, db: Db) -> Result<impl warp::Reply, Infallible> {
        println!("job status for id {:?}", opts.id.ok_or(0));
        let map = db.lock().await;
        let id = match opts.id {
            Some(i) => i,
            None => -1,
        };
        if id < 0 {
            return Ok(warp::reply::with_status(warp::reply::json(&()), StatusCode::BAD_REQUEST));
        }
        if !map.contains_key(&id) {
            return Ok(warp::reply::with_status(warp::reply::json(&()), StatusCode::NOT_FOUND));
        }
        let job = map.get(&id).unwrap().clone();
        let json = warp::reply::json(&job);
        return Ok(warp::reply::with_status(json, StatusCode::OK));
    }

    pub async fn job_update(id: i32, update: JobUpdate, db: Db) -> Result<impl warp::Reply, Infallible> {
        println!("job update: {:?}", update);
        let mut map = db.lock().await;
        if !map.contains_key(&id) {
            return Ok(StatusCode::NOT_FOUND);
        }
        let mut model = map.get(&id).unwrap().clone();
        model.finished = true;
        model.exec_output = update.exec_output;
        model.finished_at = update.finished_at;
        println!("updated to: {:?}", model.clone());
        map.insert(id, model);
        return Ok(StatusCode::OK);
    }

    // todo check if the worker should really send it's own ip address over the connection (is it necessary?)
    pub async fn register_worker(worker: Worker, worker_index: WorkerIndex, addr: Option<SocketAddr>) -> Result<impl warp::Reply, Infallible> {
        println!("registering worker: {:?}", worker);
        let ip: IpAddr = match addr {
            Some(x) => x.ip().clone(),
            None => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        };
        let mut to_insert = worker.clone();
        to_insert.ip_addr = ip.to_string();
        let mut index = worker_index.lock().await;
        index.insert(ip, to_insert);
        Ok(StatusCode::OK)
    }

    pub fn match_worker(cpus: i32, memory: i32, worker_index: MutexGuard<HashMap<IpAddr, Worker>>) -> IpAddr {
        match worker_index.keys().next() {
            Some(&ip) => ip,
            None => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        }
    }

    fn next_key(db: &MutexGuard<HashMap<i32, JobModel>>) -> i32 {
        // let mut keys: Vec<i32> = db.clone().into_keys().collect();
        // println!("keys: {:?}", keys);
        // keys.sort_unstable();
        return 0;
        // return *keys.get(keys.len() - 1);
    }
}