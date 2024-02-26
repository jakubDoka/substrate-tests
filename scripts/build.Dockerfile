# This is the build stage for Polkadot. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production as builder
WORKDIR /polkadot

# Args with default values
ARG IMAGE=chain:latest
ARG CHAIN=local
ARG VOLUME=/data
# ARG CHAIN_SPEC="${VOLUME}/${CHAIN}_spec_raw.json"
ARG NODE_BIN=/opt/node-template

COPY . /polkadot

RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the Polkadot binary."
# FROM docker.io/parity/base-bin:latest
FROM ubuntu:22.04

# Re-sourcing args from previous image
ARG IMAGE=chain:latest
ARG CHAIN=local
ARG VOLUME=/data
# ARG CHAIN_SPEC="${VOLUME}/${CHAIN}_spec_raw.json"
ARG NODE_BIN=/opt/node-template

COPY --from=builder /polkadot/target/release/node-template ${NODE_BIN}

EXPOSE 30333 9933 9944 9615

# /data is the volume where persistent data is stored,
# such as the chain state, and the chain specification file (local_spec_raw.json)
RUN mkdir ${VOLUME}
RUN ${NODE_BIN} build-spec \
  --raw \
  --disable-default-bootnode \
  > "${VOLUME}/${CHAIN}_spec_raw.json"
VOLUME ${VOLUME}

CMD ["/bin/bash"]
