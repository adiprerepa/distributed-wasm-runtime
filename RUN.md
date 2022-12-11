# Running the Project

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
