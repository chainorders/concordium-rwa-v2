# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="bookworm"
ARG CARGO_HOME="/usr/local/cargo"
ARG RUSTUP_HOME="/usr/local/rustup"

FROM mcr.microsoft.com/vscode/devcontainers/rust:1-${VARIANT}
ENV CARGO_HOME=${CARGO_HOME}
ENV RUSTUP_HOME=${RUSTUP_HOME}

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update \
   && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install clang lld \
   && apt-get autoremove -y && apt-get clean -y
RUN apt install build-essential libpq-dev
RUN rustup update \
&& rustup component add rustfmt \
&& rustup target add wasm32-unknown-unknown \
&& rustup toolchain install nightly-2023-04-01 \
&& rustup component add --toolchain nightly-2023-04-01 rustfmt \
&& cargo install --locked cargo-concordium

ADD "https://distribution.concordium.software/tools/linux/concordium-client_6.3.0-1" /usr/local/bin/concordium-client
RUN chmod +x /usr/local/bin/concordium-client
COPY --chmod=0755 bootstrap.sh /bootstrap.sh
