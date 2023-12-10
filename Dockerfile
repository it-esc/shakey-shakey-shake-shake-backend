FROM rust:1.71.0-slim as builder

WORKDIR /app

ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'

COPY . .

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update
RUN apt-get install -y musl-tools musl-dev build-essential gcc-x86-64-linux-gnu

RUN rustup target add x86_64-unknown-linux-musl && cargo build --release --target x86_64-unknown-linux-musl && mv target/x86_64-unknown-linux-musl/release/leon bin

FROM scratch as main

WORKDIR /app

COPY --from=builder /app/bin .

CMD [ "/app/bin" ]
