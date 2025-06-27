# Stage 1: Build
FROM rustlang/rust:nightly AS builder

# Enable git-fetch-with-cli (fixes GitHub clone auth issues in Docker)
RUN mkdir -p /root/.cargo && echo '[net]\ngit-fetch-with-cli = true' > /root/.cargo/config.toml

# Install system dependencies including libclang
RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    libz-dev \
    clang \
    libclang-dev \
    llvm-dev \
    llvm
# Set working directory
WORKDIR /app

# Copy entire workspace (to have access to all members like ffi/)
COPY . .

# Build only the proof-engine crate (avoid rebuilding workspace)
RUN cargo build --release -p proof-engine

# Stage 2: Runtime
FROM gcr.io/distroless/cc-debian12

# Set working directory
WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder /app/target/release/proof-engine .

# Run the binary
ENTRYPOINT ["./proof-engine"]
