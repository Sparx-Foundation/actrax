FROM docker.io/library/rust:slim-bullseye AS builder
LABEL authors="arteii"

WORKDIR /actrax

COPY . .

RUN cargo build --release --locked

RUN rm -rf target/release/build \
    && rm -rf target/release/deps \
    && rm -rf target/release/incremental \
    && rm -rf target/release/.fingerprint

FROM docker.io/library/alpine:latest AS runner

RUN apk add --no-cache libc6-compat

COPY --from=builder /actrax/target/release/actrax /usr/local/bin/actrax

EXPOSE 8000

CMD ["actrax"]
