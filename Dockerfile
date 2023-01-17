FROM rust:1.66-slim-bullseye as builder
RUN apt-get update \
 && apt-get install -y libcrypt1-dev libssl-dev g++ pkg-config \
 && rm -fr /var/lib/apt/lists/* \
 && mkdir -p /tmp/shiro-wallet
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
ARG BITCOIN_NETWORK_NAME=testnet

RUN apt-get update \
 && apt-get install -y libcrypt1 libssl1.1 libstdc++6 \
 && rm -fr /var/lib/apt/lists/* \
 && mkdir -p /tmp/shiro-wallet
COPY --from=builder /usr/local/cargo/bin/shiro-backend /usr/local/bin/shiro-backend
env BITCOIN_NETWORK_NAME=${BITCOIN_NETWORK_NAME}
CMD ["shiro-backend"]
