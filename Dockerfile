FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --no-default-features

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/drawio-tools /usr/local/bin/
ENTRYPOINT ["drawio-tools"]
