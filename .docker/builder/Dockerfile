FROM ubuntu:20.10

RUN apt-get update
# Base toolchain for develop
RUN apt-get install -y --no-install-recommends \
        jq \
        netcat \
        ca-certificates \
        gcc \
        libc6-dev \
        pkg-config \
        curl \
        wget \
        binaryen \
        wabt \
        llvm \
        libssl-dev \
        make \
        clang \
        git

# Setup user
RUN groupadd --gid 1000 develop ;\
    useradd --uid 1000 --gid develop --shell /bin/bash --create-home develop

# Rust for build and test contract
COPY --from=rust:1.54.0 /usr/local/cargo /home/develop/.cargo
COPY --from=rust:1.54.0 /usr/local/rustup /home/develop/.rustup
ENV RUSTUP_HOME=/home/develop/.rustup \
    CARGO_HOME=/home/develop/.cargo \
    PATH=/home/develop/.cargo/bin:$PATH \
    RUST_VERSION=1.54.0
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add clippy
RUN rustup component add rustfmt
RUN rustup component add rust-src
RUN cargo install cargo-audit --features=fix
RUN cargo install wasm-snip wasm-gc

RUN mkdir /code
RUN chown develop:develop -R /code /home/develop
WORKDIR /code
USER develop

LABEL org.opencontainers.image.source https://github.com/depositum/dapp
