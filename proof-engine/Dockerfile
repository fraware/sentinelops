FROM rustlang/rust:nightly AS builder

# Install build dependencies
RUN mkdir -p /root/.cargo && echo '[net]\ngit-fetch-with-cli = true' >> /root/.cargo/config.toml
RUN apt-get update && apt-get install -y cmake pkg-config libz-dev

WORKDIR /app

# Copy app code
COPY . .
COPY proof-engine/ .
COPY ../lean/ffi ./lean/ffi

# Build
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/proof-engine .
ENTRYPOINT ["./proof-engine"]
