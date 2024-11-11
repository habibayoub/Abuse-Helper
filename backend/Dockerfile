# syntax=docker/dockerfile:1.4
FROM rust:buster AS base

ENV USER=root
ENV RUST_LOG=debug

WORKDIR /code
RUN cargo init
COPY Cargo.toml /code/Cargo.toml
RUN cargo fetch
COPY . /code

FROM base AS development

EXPOSE 8000

CMD ["cargo", "run", "--offline"]

FROM base AS dev-envs

EXPOSE 8000
RUN <<EOF
apt-get update
apt-get install -y --no-install-recommends git build-essential cmake libssl-dev pkg-config
EOF

RUN <<EOF
useradd -s /bin/bash -m vscode
groupadd docker
usermod -aG docker vscode
EOF
# install Docker tools (cli, buildx, compose)
COPY --from=gloursdocker/docker / /

CMD ["cargo", "run", "--offline"]

FROM base AS builder

RUN cargo build --release --offline

FROM debian:buster-slim

ENV RUST_LOG=debug

WORKDIR /code

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /code/target/release/abuse_helper ./

EXPOSE 8000

CMD ["./abuse_helper"]
