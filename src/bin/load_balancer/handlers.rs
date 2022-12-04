
mod handlers {
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::SystemTime;
    use rand::Rng;
    use distributed_wasm_runtime::modules::{CreateJobResponse, Db, Job, JobModel, JobOptions, JobUpdate, Worker, WorkerIndex};
    use rand::rngs::ThreadRng;
    use tokio::sync::MutexGuard;
    use warp::http::StatusCode;


    pub async fn create_job(job: Job, db: Db) -> Result<impl warp::Reply, Infallible> {
        println!("create job: {:?}", job);
        let mut map: MutexGuard<HashMap<i32, JobModel>> = db.lock().await;
        let mut rng: ThreadRng = rand::thread_rng();
        let mut id: i32 = 0;
        let base: i32 = 10;
        loop {
            id = rng.gen_range(0..base.pow(8));
            if !map.contains_key(&id) {
                break;
            }
        }
        let started_at = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        map.insert(id, JobModel{
            job: job.clone(),
            finished: false,
            finished_at: 0,
            started_at: started_at.as_secs(),
            job_id: id,
            exec_output: String::from(""),
        });
        let json = warp::reply::json(&CreateJobResponse{ id });
        // call worker
        Ok(warp::reply::with_status(json, StatusCode::OK))
    }

    pub async fn status_job(opts: JobOptions, db: Db) -> Result<impl warp::Reply, Infallible> {
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
        to_insert.ip_addr = ip.into_string();
        let mut index = worker_index.lock().await;
        index.insert(ip, to_insert);
        Ok(StatusCode::OK)
    }
}