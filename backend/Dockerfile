# syntax=docker/dockerfile:1.4
FROM rust:buster AS base

ENV USER=root
ENV RUST_LOG=debug
ENV RUSTFLAGS="-C target-feature=+fp16"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C target-feature=+fp16"
WORKDIR /code
RUN cargo init
COPY Cargo.toml /code/Cargo.toml
RUN cargo fetch
COPY . /code

FROM base AS development

EXPOSE 8000

CMD [ "cargo", "run", "--offline" ]

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
CMD [ "cargo", "run", "--offline" ]

FROM base AS builder

RUN cargo build --release --offline

FROM debian:buster-slim

ENV RUST_LOG=debug
ENV RUSTFLAGS="-C target-feature=+fp16"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C target-feature=+fp16"

EXPOSE 8000

COPY --from=builder /code/target/release/abuse-helper /abuse-helper

CMD [ "/abuse-helper" ]
