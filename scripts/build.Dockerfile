# First stage builds the chain node
FROM ubuntu:latest AS builder
WORKDIR /workspace

RUN apt-get update\
 && apt-get install -y git clang curl libssl-dev llvm libudev-dev protobuf-compiler make

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable\
 && rustup install 1.65.0\
 && rustup install nightly-2024-01-31\
 && rustup target add wasm32-unknown-unknown --toolchain stable\
 && rustup target add wasm32-unknown-unknown --toolchain nightly-2024-01-31

# COPY target/release/node-template /workspace/target/release
# COPY scripts/docker_start.sh /workspace/scripts
# COPY scripts/testnet_spec.json /workspace/scripts
COPY . /workspace
RUN cd /workspace && cargo build --release

# Final stage is the chain node
FROM ubuntu:latest

COPY --from=builder /workspace/target/release/node-template /opt
COPY --from=builder /workspace/scripts/docker_start.sh /opt
COPY --from=builder /workspace/scripts/testnet_spec.json /opt
CMD ["/bin/bash", "/opt/docker_start.sh"]
