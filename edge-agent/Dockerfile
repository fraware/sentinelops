# ---------- build stage -------------------------------------------------
FROM rust:1.78 AS builder

# Install required system packages
RUN apt-get update && apt-get install -y cmake pkg-config libz-dev

WORKDIR /app
COPY . .
RUN cargo build --release

# ---------- runtime stage -----------------------------------------------
FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/edge-agent .
ENTRYPOINT ["./edge-agent"]
