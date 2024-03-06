#!/bin/bash

source "/data/variables.sh"

start_validator() {
  /opt/node-template key insert \
    --base-path /opt/node_data \
    --chain $CHAIN_SPEC \
    --scheme Sr25519 \
    --suri "$SEED" \
    --key-type aura

  /opt/node-template key insert \
    --base-path /opt/node_data \
    --chain $CHAIN_SPEC \
    --scheme Ed25519 \
    --suri "$SEED" \
    --key-type gran

  /opt/node-template \
    --base-path /opt/node_data \
    --chain $CHAIN_SPEC \
    --port $PORT \
    --rpc-port $RPC_PORT \
    --rpc-cors all \
    --validator \
    --allow-private-ip \
    --name $NAME
}

start_rpc() {
  /opt/node-template \
    --base-path /opt/node_data \
    --chain $CHAIN_SPEC \
    --rpc-port $RPC_PORT \
    --rpc-external --rpc-methods safe \
    --rpc-cors all \
    --allow-private-ip \
    --state-pruning archive \
    --name "rpc"
}

if [[ "$VALIDATOR" == "TRUE" ]]; then
  echo " --- Starting as validator."
  start_validator
else
  echo " --- Starting as rpc node."
  start_rpc
fi

