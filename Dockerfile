# --- build ---
FROM rust:1.92-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# --- runtime ---
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-prefork-poll-http-server /usr/local/bin/rust-prefork-poll-http-server

ENV PORT=8080
EXPOSE 8080

CMD ["rust-prefork-poll-http-server"]
