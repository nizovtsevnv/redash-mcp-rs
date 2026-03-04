FROM rust:1.85-slim AS builder

WORKDIR /app

# Cache dependencies by copying manifests first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Build the actual application
COPY . .
RUN touch src/main.rs && cargo build --release

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/redash-mcp /

ENTRYPOINT ["/redash-mcp"]
