
FROM rust:1.67.1-alpine3.17 as builder
WORKDIR /usr/src
RUN rustup target add x86_64-unknown-linux-musl
RUN apk add --no-cache openssl-dev openssl-libs-static musl-dev

WORKDIR /root/build/
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release
