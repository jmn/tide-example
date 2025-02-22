FROM rust:1.48 AS planner
WORKDIR /app
# We only pay the installation cost once,
# it will be cached from the second build onwards
# To ensure a reproducible build consider pinning
# the cargo-chef version with `--version X.X.X`
RUN cargo install cargo-chef
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:1.48 AS cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.48 AS builder
WORKDIR /app
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
# Build our application, leveraging the cached deps!
#ENV SQLX_OFFLINE true
ENV TIDE_PORT 9090

RUN cargo build --release --bin tide-example

FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tide-example tide-example
COPY templates templates
COPY db db
EXPOSE 9090

ENTRYPOINT ["./tide-example"]
