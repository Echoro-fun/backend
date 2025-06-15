FROM rust:slim-bullseye AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && apt-get clean

WORKDIR /app
COPY --from=builder /app/target/release/echoro-backend .

EXPOSE 8080
CMD ["./echoro-backend"]
