# syntax=docker/dockerfile:1

# Stage 1: Build
FROM rust:1-bookworm AS builder

WORKDIR /app

# Copy manifests first to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build and cache dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Remove dummy source and copy actual source code
RUN rm -rf src
COPY src ./src

# Touch main.rs to ensure cargo rebuilds the application
RUN touch src/main.rs
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Create the data directory for mounting NeTEx files
RUN mkdir -p /app/data

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/netex-indexer /app/netex-indexer

# Run the indexer
ENTRYPOINT ["/app/netex-indexer"]
