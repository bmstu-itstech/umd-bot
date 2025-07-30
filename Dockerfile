FROM rust:latest AS builder
LABEL authors="zhikh"

WORKDIR /usr/src/bot

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates openssl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/bot/target/release/umd-bot /usr/local/bin/umd_bot

CMD ["umd_bot"]
