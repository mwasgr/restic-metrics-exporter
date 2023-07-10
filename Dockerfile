FROM rust:latest as build

WORKDIR /app

COPY src src
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build -r

FROM alpine:latest
RUN apk add restic

WORKDIR /app
EXPOSE 9184

COPY --from=build /app/target/release .

ENTRYPOINT ["/app/restic-metrics-exporter"]