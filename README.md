# Distributed Wasm Runtime

Members: Aditya Prerepa (prerepa2), Shashwat Mundra, Aryan Arora

## Project Introduction

This project is essentially a generalized distributed runtime for [WebAssembly](https://webassembly.org/). When carried out, a user of this project would be able to give a Rust, C++, or Go function on a webassembly runtime hosted on a distributed worker system. The rust/cpp/go function would be compiled to webassembly on the server, and that webassembly would be sent to the appropriate worker (based on cpu/gpu/memory/timelimit requests). That worker would then run the webassembly, and send the returned result to the server, and the server would forward that response to the client. The server would also returned a copy of the compiled WebAssembly for future use.

The value here lies in the fact developers of these abstract functions don't necessarily have to worry about the language they are writing their functions in, the environment in which they are run, or the scaling of compute resources to match the function. This project allows developers to just write abstract functions without having to worry about standardization across language at all. And because the server returns a copy of the compiled WebAssembly, developers of these functions can reproduce what was run locally as well.

## Technical Overview

Working with WebAssembly and building Distributed Systems are both not trivial. There are a few major components that we have to get right:

- Cross-Compiling with C++, Go, or Rust input files to a standard WebAssembly format.
- Finding the most effective WebAssembly Runtime to use for the workers.
- Building the client CLI crate that interacts with the server.
- Building a server that has the ability to compile to WebAssembly and that has the ability to load-balance and match workers.
- Building generalized worker software that is able to properly run WebAssembly and is able to utilize compute resources as allocated.

The architecture of this project would look something like:

![Architecture](https://github.com/adiprerepa/distributed-wasm-runtime/blob/main/cs128h%20architecture.jpg)



