# ---------- Build ----------
FROM rust:1.91-slim-bookworm AS build
WORKDIR /app

# Cache phụ thuộc: build một main giả trước để lớp này chỉ dựng lại khi Cargo.toml / Cargo.lock đổi.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src target/release/deps/foodrandom_api* target/release/foodrandom-api

COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --locked

# ---------- Runtime ----------
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home /app foodrandom
WORKDIR /app

COPY --from=build /app/target/release/foodrandom-api /usr/local/bin/foodrandom-api
COPY seed ./seed

USER foodrandom
ENV HOST=0.0.0.0 \
    PORT=8180 \
    LOG_FORMAT=json \
    SEED_DIR=/app/seed/foods
EXPOSE 8180

HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS http://127.0.0.1:8180/health || exit 1

ENTRYPOINT ["/usr/local/bin/foodrandom-api"]
