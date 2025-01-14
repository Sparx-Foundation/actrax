FROM rust:latest AS builder
LABEL authors="arteii"

WORKDIR /server

COPY . .

RUN cargo build --release --bin server

RUN rm -rf target/release/build \
    && rm -rf target/release/deps \
    && rm -rf target/release/incremental \
    && rm -rf target/release/.fingerprint

FROM ubuntu:latest as runner

COPY --from=builder /server/target/release/server /usr/local/bin/server

EXPOSE 8000

CMD ["server"]
