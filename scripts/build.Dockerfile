# First stage builds the chain node
FROM ubuntu:latest AS builder

RUN apt-get update\
 && apt-get install -y git clang curl libssl-dev llvm libudev-dev protobuf-compiler make

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable\
 && rustup update\
 && rustup update nightly\
 && rustup target add wasm32-unknown-unknown --toolchain nightly

COPY . /workspace
# Takes around 10 minutes to build
RUN cd /workspace && cargo build --release

# Final stage is the chain node
FROM ubuntu:latest

COPY --from=builder /workspace/target/release/node-template /opt/node-template
COPY docker_start.sh /opt/docker_start.sh
CMD ["/bin/bash", "/opt/docker_start.sh"]

