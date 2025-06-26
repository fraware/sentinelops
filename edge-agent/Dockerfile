# ---------- build stage -------------------------------------------------
FROM rust:1.78 AS builder
WORKDIR /app
COPY edge-agent/ .
RUN cargo build --release

# ---------- runtime stage -----------------------------------------------
FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/edge-agent .
ENTRYPOINT ["./edge-agent"]
