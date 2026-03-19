FROM rust:1.75-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release --no-default-features

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/drawio-tools /usr/local/bin/
ENTRYPOINT ["drawio-tools"]
