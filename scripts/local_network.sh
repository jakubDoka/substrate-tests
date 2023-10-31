#!/bin/bash
script_dir="$(dirname "$(readlink -f "$0")")"
cd "${script_dir}/.."

tmp="scripts/.tmp"
mkdir $tmp
ssnode="./scripts/.tmp/node-template"
image="ghe0/chain_testnet:latest"

cargo build --release
mv ./target/release/node-template $ssnode

cd "$script_dir"
docker build -t $image .
cd "${script_dir}/.."

docker rm -f ghe0_node1 ghe0_node2 ghe0_node3 ghe0_node4 ghe0_node5 ghe0_rpc

docker run -d --name ghe0_node1 \
  --env VALIDATOR=TRUE \
  --env NODE_NAME=Alice --env "SEED=//Alice" \
  --env "CHAIN=local" \
  $image

docker run -d --name ghe0_node2 \
  --env VALIDATOR=TRUE \
  --env NODE_NAME=Bob --env "SEED=//Bob" \
  --env "CHAIN=local" \
  $image

docker run -d --name ghe0_node3 \
  --env VALIDATOR=TRUE \
  --env NODE_NAME=Charlie --env "SEED=//Charlie" \
  --env "CHAIN=local" \
  $image

docker run -d --name ghe0_node4 \
  --env VALIDATOR=TRUE \
  --env NODE_NAME=Bob --env "SEED=//Dave" \
  --env "CHAIN=local" \
  $image

docker run -d --name ghe0_node5 \
  --env VALIDATOR=TRUE \
  --env NODE_NAME=Charlie --env "SEED=//Eve" \
  --env "CHAIN=local" \
  $image

docker run -d --name ghe0_rpc \
  --publish 9944:9944 \
  --env "CHAIN=local" \
  $image
