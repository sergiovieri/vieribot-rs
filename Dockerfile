FROM rust:1.65-buster as builder

WORKDIR /bot

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
  echo "// dummy" >src/lib.rs && \
  cargo build --release && \
  rm -r src

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get upgrade -y && apt-get install -y ca-certificates && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

COPY --from=builder /bot/target/release/vieribot-rs /usr/local/bin/vieribot-rs
COPY Cargo.lock /

CMD ["/usr/local/bin/vieribot-rs"]
