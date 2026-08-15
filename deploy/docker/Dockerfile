FROM rust:1.80-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p prism-cli

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/prism /usr/local/bin/prism
COPY data/samples /data/samples
CMD ["prism", "version"]
