FROM rust:alpine as builder

WORKDIR /usr/src/message-service

RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./

COPY src/ ./src/

RUN cargo build --release

FROM alpine:latest
WORKDIR /usr/app

RUN apk add --no-cache bash

COPY --from=builder /usr/src/message-service/target/release/message-service .
COPY check_cluster_healthy.sh .

RUN chmod +x check_cluster_healthy.sh

EXPOSE 3000

ENTRYPOINT ["./check_cluster_healthy.sh"]