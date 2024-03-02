# This is the build stage for Polkadot. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production as builder
WORKDIR /polkadot

# Build time variables with default values
ARG NAME=node
ARG IMAGE=chain:latest
ARG CHAIN=local
ARG VOLUME=/data
ARG NODE_BIN=/opt/node-template

# Runtime variables, used by the command `docker run`
# this default value can be replaced by using `--env` flag
ENV SURI="narrow use math topple stage produce top satoshi rapid satisfy half naive"

RUN mkdir -p /polkadot/target/release && mkdir -p /polkadot/scripts
COPY ./target/release/node-template /polkadot/target/release
COPY ./scripts/${CHAIN}_spec.json /polkadot/scripts
# COPY . /polkadot
# RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the Polkadot binary."
# FROM docker.io/parity/base-bin:latest
FROM archlinux:latest

# Re-sourcing args from previous builder image
ARG NAME=node
ARG IMAGE=chain:latest
ARG CHAIN=local
ARG VOLUME=/data
ARG NODE_BIN=/opt/node-template

# Resourcing envs from previous builder image
ENV SURI="narrow use math topple stage produce top satoshi rapid satisfy half naive"

RUN mkdir ${VOLUME}
COPY --from=builder /polkadot/target/release/node-template ${NODE_BIN}
COPY --from=builder /polkadot/scripts/${CHAIN}_spec.json "${VOLUME}/${CHAIN}_spec.json"

# /data is the volume where persistent data is stored,
# such as the chain state, and the chain specification file (local_spec_raw.json)
# RUN ${NODE_BIN} build-spec \
#   --raw \
#   --chain "${VOLUME}/tmp.json" \
#   --disable-default-bootnode \
#   > "${VOLUME}/${CHAIN}_spec_raw.json"

# generate aura and grandpa keys
RUN ${NODE_BIN} key insert \
  --base-path ${VOLUME} \
  --chain "${VOLUME}/${CHAIN}_spec.json" \
  --scheme Sr25519 \
  --suri "${SURI}" \
  --key-type aura

RUN ${NODE_BIN} key insert \
  --base-path ${VOLUME} \
  --chain "${VOLUME}/${CHAIN}_spec.json" \
  --scheme Ed25519 \
  --suri "${SURI}" \
  --key-type gran

VOLUME ${VOLUME}
EXPOSE 30333 9933 9944 9615

# /opt/node-template build-spec \
#   --chain "/data/local_spec_raw.json" \
#   --disable-default-bootnode \
#   > "/data/local_spec.json"

CMD ["/bin/bash"]
