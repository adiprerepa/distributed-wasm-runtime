# Distributed Wasm Runtime

Members: Aditya Prerepa (prerepa2), Shashwat Mundra (mundra3), Aryan Arora (aarora14)

### Slight Disclaimer

This README is split up into two parts: the current, functional section which showcases the features of this project and its overarching goals, and the "assignment" section (Archive), which was written before the project was create at the request of the entities administering the course. It might be a fun exercise to see what was promised in this project, and what was actually delivered.


## Overview

This project is an exercise in using WebAssembly as the runtime in a Distributed Worker System. All components were built from scratch, without best practices, so this is not meant to be a reliable, fault-tolerant system. For something more production-ready, check out something like [Codalab Worksheets](https://worksheets.codalab.org/). This system allows users to provide a rust source, for this rust source to be compiled to WebAssembly, matched to an appropriate worker (based on compute requirements), and for the user to then receive output of the job along with generalized run statistics. The difference between this and other distributed worker systems is that conventionally, jobs are executed in a container. The aim of WebAssembly however, is to replace these containers for a more performant and reliable system. So, it naturally makes sense to use WebAssembly for a case like this.

### Components

There are three main components in this project:
- the command line interface (`/src/bin/cli`)
- the load balancer/wasm compiler (`/src/bin/load_balancer`)
- the worker (`/src/bin/worker`)

The command line interface makes its requests to the load balancer, providing the rust source, along with resource (cpu, memory) requests. The load balancer receives this job, and compiles it to WebAssembly using the `wasm32-wasi` runtime. The load balancer then matches an appropriate worker from the worker pool, and sends the WebAssembly binary to that worker. The workers only function is to execute this WebAssembly, and send back the result. The load balancer then does some statistics aggregation about the run, and publishes the result to an (in-memory) database. The command line interface can then pull these results.

#### A note about Worker Discovery

When workers come online, they advertise their presence to the load balancer (which should already be online). This is an extremely crude "Discovery Service" which really only supports worker addition. By default, the workers assume the load balancer runs at localhost, and the load balancer assumes the workers run at localhost. Additionally, the way the load balancer indexes worker is by IP Address, meaning only one instance of the worker can run per address, or the worker advertisement will not only fail, but evict the other instance of the worker from the index.

In the most basic example, the command line, load balancer, and (singular) worker all run on localhost. You can in theory add multiple workers on other IP Addresses, but for the purposes of demo, we will be showcasing a "fully-local" approach.

### Running (fully local)

This setup was tested with Ubuntu 22.04.

Clone the repository locally with:

```bash
git clone git@github.com:adiprerepa/distributed-wasm-runtime.git
```

Ideally, follow these steps in this order.

#### Bringing up a worker

Bringing up a worker requires no dependencies.

```bash
cargo run --bin worker
```

This runs an HTTP server synchronously, so leave this process running.

#### Bringing up the Load Balancer

Bringing up the Load Balancer requires you to have `rustc/rustup` installed, along with the `wasm32-wasi` runtime installed. You can install this runtime with:

```bash
rustup target add wasm32-wasi
```

This target is required because the WebAssembly compilation is run from the kernel space, whereas the execution on the worker is run in the user space as a statically linked library.

To bring up the load balancer:

```bash
cargo run --bin load_balancer
```

#### Calling the service
Say you have a `hello.rs` in your working directory with the contents:
```rust
use std::{thread, time};
fn main() {
	thread::sleep(time::Duration::new(5, 0));
	println!("job done, hello!");
}
```

You can submit a job with:

```bash
cargo run --bin cli run hello.rs --cpus 1 --memory-mb 1000 --job-name sleep
```

You should get output like (id varies):

```bash
job running, id: 10.
```

You can pull the status of the job with:

```bash
cargo run --bin cli status --id 10
```

To then get output similar to:

```bash
job status:
	name: "sleep"
	finished: true
	running time: 6s
	execution output: "job done, hello!\n"
```

The above output was extracted from the WebAssembly execution.
## Archive

### Project Introduction

This project is essentially a generalized distributed runtime for [WebAssembly](https://webassembly.org/). When carried out, a user of this project would be able to give a Rust, C++, or Go function on a webassembly runtime hosted on a distributed worker system. The rust/cpp/go function would be compiled to webassembly on the server, and that webassembly would be sent to the appropriate worker (based on cpu/gpu/memory/timelimit requests). That worker would then run the webassembly, and send the returned result to the server, and the server would forward that response to the client. The server would also returned a copy of the compiled WebAssembly for future use.

The value here lies in the fact developers of these abstract functions don't necessarily have to worry about the language they are writing their functions in, the environment in which they are run, or the scaling of compute resources to match the function. This project allows developers to just write abstract functions without having to worry about standardization across language at all. And because the server returns a copy of the compiled WebAssembly, developers of these functions can reproduce what was run locally as well.

### Technical Overview

Working with WebAssembly and building Distributed Systems are both not trivial. There are a few major components that we have to get right:

- Cross-Compiling with C++, Go, or Rust input files to a standard WebAssembly format.
- Finding the most effective WebAssembly Runtime to use for the workers.
- Building the client CLI crate that interacts with the server.
- Building a server that has the ability to compile to WebAssembly and that has the ability to load-balance and match workers.
- Building generalized worker software that is able to properly run WebAssembly and is able to utilize compute resources as allocated.

The architecture of this project would look something like:

![Architecture](https://github.com/adiprerepa/distributed-wasm-runtime/blob/main/cs128h%20architecture.jpg)


### Checkpoints for the PM

Checkpoint 1: Done with the CLI, proof of concept WASM compilation for all three supported languages.

Checkpoint 2: Done with server & WASM compilation workflow, networking infrastructure underway and WASM execution on worker POC.

### Possible Challenges

NONE!

Realistically, there are many things that could go wrong, since WebAssembly and Distributed Systems have a lot of moving parts and are not trivial to work with. But we will transcend these challenges.

If anything, there might be problems with conveying what to run from client to worker without being too verbose, but this can be addressed by rethinking design at some point.
