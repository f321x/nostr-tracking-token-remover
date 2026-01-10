FROM ubuntu:22.04 AS build

RUN apt-get update && apt-get -y upgrade
RUN apt-get -y install \
    gcc \
    curl \
    musl-tools

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY . /tracking

WORKDIR /tracking

RUN rustup target add x86_64-unknown-linux-musl

RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine AS binaries

RUN apk add --no-cache libgcc

COPY --from=build /tracking/target/x86_64-unknown-linux-musl/release/ /usr/local/bin

CMD ["nostr-tracking-token-remover"]