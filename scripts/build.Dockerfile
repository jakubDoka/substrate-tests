# This is the build stage for Polkadot. Here we create the binary in a temporary image.
# FROM docker.io/paritytech/ci-linux:production as builder
FROM ubuntu:22.04 as builder
WORKDIR /polkadot

RUN apt-get update \
 && apt-get install -y git clang curl libssl-dev llvm libudev-dev protobuf-compiler make

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable \
 && rustup update \
 && rustup update nightly \
 && rustup target add wasm32-unknown-unknown --toolchain nightly

# Build time variables with default values
ARG CHAIN=local
ARG VOLUME=/data
ARG NODE_BIN=/opt/node-template

COPY . /polkadot
RUN mkdir -p /polkadot/target/release && mkdir -p /polkadot/scripts

COPY ./scripts/${CHAIN}_spec.json /polkadot/scripts
COPY ./scripts/docker_start.sh /polkadot/scripts
COPY ./scripts/variables.sh /polkadot/scripts

# on development, if you already have the compiled binary locally,
# it's much faster to uncomment the next line, and comment the build line.
# remember to remove "target" from .dockerignore.
# COPY ./target/release/node-template /polkadot/target/release
RUN cargo build --release --locked -j 8

# FROM docker.io/parity/base-bin:latest
FROM archlinux:latest

# Re-sourcing args from previous builder image
ARG CHAIN=local
ARG VOLUME=/data
ARG NODE_BIN=/opt/node-template

RUN mkdir ${VOLUME}
COPY --from=builder /polkadot/target/release/node-template ${NODE_BIN}
COPY --from=builder /polkadot/scripts/${CHAIN}_spec.json "${VOLUME}/${CHAIN}_spec.json"
COPY --from=builder /polkadot/scripts/docker_start.sh "${VOLUME}"
COPY --from=builder /polkadot/scripts/variables.sh "${VOLUME}"

EXPOSE 30333 9933 9944 9615

VOLUME ${VOLUME}

CMD ["/bin/bash"]
