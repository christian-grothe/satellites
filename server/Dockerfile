FROM rust:latest AS builder
WORKDIR /usr/src/satellites
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get upgrade && apt-get install libc-bin libc6 && mkdir recordings
COPY --from=builder /usr/src/satellites/target/release/server /usr/local/bin
CMD ["server"]
