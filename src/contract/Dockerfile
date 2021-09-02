FROM rust:1.54.0

RUN apt-get update
RUN apt-get install -y git
RUN apt-get install -y binaryen wabt

RUN rustup default 1.54.0
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-snip wasm-gc
