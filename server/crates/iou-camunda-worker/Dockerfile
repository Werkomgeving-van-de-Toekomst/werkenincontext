# Build vanuit repository-root: docker build -f crates/iou-camunda-worker/Dockerfile .
FROM rust:1.85-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations
COPY infra/camunda/bpmn ./infra/camunda/bpmn
RUN cargo build --release -p iou-camunda-worker

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/iou-camunda-worker /usr/local/bin/iou-camunda-worker
COPY infra/camunda/bpmn /app/bpmn
ENV ZEEBE_DEPLOY_BPMN=/app/bpmn/document-pipeline.bpmn
ENTRYPOINT ["/usr/local/bin/iou-camunda-worker"]
