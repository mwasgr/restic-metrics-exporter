FROM rust:latest as build

WORKDIR /app

COPY src src
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build -r

FROM debian:latest
RUN apt-get update
RUN apt-get -y install restic

WORKDIR /app
EXPOSE 80

COPY --from=build /app/target/release/restic-metrics-exporter .

ENTRYPOINT ["./restic-metrics-exporter"]