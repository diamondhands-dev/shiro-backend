# shiro-backend
Wallet server for the RGB protocol (that is defined LNP/BP)

# Prequisite

* Electrum and bitcoin-node that sync on your bitcoin network. (`mainnet`, `testnet`, `signet`, or `regtest`).
* rgb-proxy-server.  One of the rgb-proxy-server is https://github.com/grunch/rgb-proxy-server .
* Some more shared libraries like libcrypt1.so.

And you want to develop Shiro-backend itself.

# How to run

## Docker

You shall specify `{network_name}`. (`mainnet`, `testnet`, `signet`, or `regtest`).

```
docker run -d -p 8080:8080 -e BITCOIN_NETWORK={network_name} ghcr.io/diamondhands-dev/shiro-backend:latest
```

## From sources

```
git clone https://github.com/diamondhands-dev/shiro-backend.git
cd shiro-backend
cargo install
export BITCOIN_NETWORK_NAME={network_name}
export ELECTRUM_URL=127.0.0.1:50001
export RGB_PROXY_URL=http://proxy.rgbtools.org

shiro-backend
```

Environment variables will be fixed for your runtime environment.

# How to test

## Prequisite

* Linux runs on 64bit architecture.
  * You'll get a build error on 32bit versions of Linux (like i386, armv7).
  * I've not checked yet on Windows, macOS. I guess Shiro-backend won't work with them.
* Rust/Cargo (>= 1.66.0 ).
* C++ toolchain (Shiro itself doesn't need but some dependencies require it).
* Some additional `*.a` library files. It depends your development environment.

## Steps.

```
git clone https://github.com/diamondhands-dev/shiro-backend.git
cd shiro-backend
mkdir -p /tmp/shiro-wallet
test-mocks/start_services.sh
RUST_TEST_THREADS=1 BITCOIN_NETWORK_NAME={network_name} cargo test
```

# How to release docker images

```
git tag v{semantic version}
git push origin v{semantic version}
```

A GitHub action will be build new image with tags.

You'll find the rule of `{semantic version}` is https://semver.org/

# Help wanted?

https://github.com/diamondhands-dev/shiro-backend/issues
