# This is the build stage for Polkadot. Here we create the binary in a temporary image.
# FROM docker.io/paritytech/ci-linux:production as builder
FROM ubuntu:22.04 as builder
WORKDIR /polkadot

RUN apt-get update\
 && apt-get install -y git clang curl libssl-dev llvm libudev-dev protobuf-compiler make

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable\
 && rustup update\
 && rustup update nightly\
 && rustup target add wasm32-unknown-unknown --toolchain nightly

# Build time variables with default values
ARG NAME=node
ARG IMAGE=chain:latest
ARG CHAIN=local
ARG VOLUME=/data
ARG NODE_BIN=/opt/node-template

# Runtime variables, used by the command `docker run`
# this default value can be replaced by using `--env` flag

RUN mkdir -p /polkadot/target/release && mkdir -p /polkadot/scripts
# COPY ./target/release/node-template /polkadot/target/release
COPY ./scripts/${CHAIN}_spec.json /polkadot/scripts
COPY ./scripts/docker_start.sh /polkadot/scripts
COPY ./scripts/variables.sh /polkadot/scripts

COPY . /polkadot
RUN cargo build --release --locked -j 8

# This is the 2nd stage: a very small image where we copy the Polkadot binary."
# FROM docker.io/parity/base-bin:latest
FROM archlinux:latest

# Re-sourcing args from previous builder image
ARG NAME=node
ARG IMAGE=chain:latest
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
