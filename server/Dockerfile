FROM rust:bookworm AS builder
WORKDIR /usr/src/satellites
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN mkdir /recordings
COPY --from=builder /usr/src/satellites/target/release/server /usr/local/bin/
WORKDIR /
CMD ["server"]
