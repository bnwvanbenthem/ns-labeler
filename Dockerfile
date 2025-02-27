# Stage 1: Build the Rust binary with musl
FROM rust:1.81 AS builder
# Install musl-dev for musl-gcc
RUN apt-get update && apt-get install -y musl-dev musl-tools
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/tagging-operator
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Create the runtime image
FROM alpine:3.18
RUN apk add --no-cache ca-certificates
COPY --from=builder /usr/src/tagging-operator/target/x86_64-unknown-linux-musl/release/tagging_operator /usr/local/bin/tagging_operator
ENTRYPOINT ["/usr/local/bin/tagging_operator"]