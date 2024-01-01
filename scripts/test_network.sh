#!/bin/bash
script_dir="$(dirname "$(readlink -f "$0")")"
cd "${script_dir}/.."

spec="scripts/testnet_spec.json"
tmp="scripts/.tmp"
mkdir $tmp
spec_raw="$tmp/testnet_spec_raw.json"
ssnode="./scripts/.tmp/node-template"
image="ghe0/chain_testnet:latest"

# public keys for these seeds have been added to the spec
node1_seed="narrow use math topple stage produce top satoshi rapid satisfy half naive"
node2_seed="lesson online video chapter match pig response inner degree brown often cover"
node3_seed="female city jewel name sausage riot zebra lunch access message buyer gold"
node4_seed="galaxy bundle tuition kite believe opinion page energy wine live farm donkey"
node5_seed="sing earth victory dove tag siege cereal embody scheme grant swear level"

cargo build --release
mv ./target/release/node-template $ssnode

cd "$script_dir"
docker build -t $image -f move.Dockerfile .
cd "${script_dir}/.."

$ssnode build-spec --chain=$spec --raw --disable-default-bootnode > "$spec_raw"

docker rm -f ghe0_node1 ghe0_node2 ghe0_node3 ghe0_node4 ghe0_node5 ghe0_rpc

docker run -d --name ghe0_node1 \
  --env "VALIDATOR=TRUE" \
  --env "NODE_NAME=node1" --env "SEED=$node1_seed" \
  --env "CHAIN=/opt/testnet_spec_raw.json" \
  --volume "$(pwd)/$spec_raw:/opt/testnet_spec_raw.json:ro" \
  $image

docker run -d --name ghe0_node2 \
  --env "VALIDATOR=TRUE" \
  --env "NODE_NAME=node2" --env "SEED=$node2_seed" \
  --env "CHAIN=/opt/testnet_spec_raw.json" \
  --volume "$(pwd)/$spec_raw:/opt/testnet_spec_raw.json:ro" \
  $image

docker run -d --name ghe0_node3 \
  --env "VALIDATOR=TRUE" \
  --env "NODE_NAME=node3" --env "SEED=$node3_seed" \
  --env "CHAIN=/opt/testnet_spec_raw.json" \
  --volume "$(pwd)/$spec_raw:/opt/testnet_spec_raw.json:ro" \
  $image

docker run -d --name ghe0_node4 \
  --env "VALIDATOR=TRUE" \
  --env "NODE_NAME=node4" --env "SEED=$node4_seed" \
  --env "CHAIN=/opt/testnet_spec_raw.json" \
  --volume "$(pwd)/$spec_raw:/opt/testnet_spec_raw.json:ro" \
  $image

docker run -d --name ghe0_node5 \
  --env "VALIDATOR=TRUE" \
  --env "NODE_NAME=node4" --env "SEED=$node5_seed" \
  --env "CHAIN=/opt/testnet_spec_raw.json" \
  --volume "$(pwd)/$spec_raw:/opt/testnet_spec_raw.json:ro" \
  $image

docker run -d --name ghe0_rpc \
  --publish 9944:9944 \
  --env "CHAIN=/opt/testnet_spec_raw.json" \
  --volume "$(pwd)/$spec_raw:/opt/testnet_spec_raw.json:ro" \
  $image
