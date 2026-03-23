# Build vanuit repository-root:
#   docker build -f crates/iou-ai-service/Dockerfile .
FROM rust:1.85-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p iou-ai-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/iou-ai-service /usr/local/bin/iou-ai-service
EXPOSE 8090
ENV IOU_AI_SERVICE_HOST=0.0.0.0
ENV IOU_AI_SERVICE_PORT=8090
ENTRYPOINT ["/usr/local/bin/iou-ai-service"]
