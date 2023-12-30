#!/bin/bash

start_validator() {
  /opt/node-template key insert \
    --base-path /opt/node_data \
    --chain $CHAIN \
    --scheme Sr25519 \
    --suri "$SEED" \
    --key-type aura

  /opt/node-template key insert \
    --base-path /opt/node_data \
    --chain $CHAIN \
    --scheme Ed25519 \
    --suri "$SEED" \
    --key-type gran

  /opt/node-template \
    --base-path /opt/node_data \
    --chain $CHAIN \
    --port 30333 \
    --rpc-port 9944 \
    --rpc-cors all \
    --validator \
    --allow-private-ip \
    --name $NODE_NAME
}

start_rpc() {
  /opt/node-template \
    --base-path /opt/node_data \
    --chain $CHAIN \
    --rpc-port 9944 \
    --rpc-external --rpc-methods safe \
    --rpc-cors all \
    --allow-private-ip \
    --state-pruning archive \
    --name "rpc_node"
}

if [[ "$VALIDATOR" == "TRUE" ]]; then
  echo " --- Starting as validator."
  start_validator
else
  echo " --- Starting as rpc node."
  start_rpc
fi
