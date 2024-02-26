#!/bin/bash
#
# Spawn all the node containers.
# To update the chain spec of the nodes, after running this script: `./update_spec.sh`

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/variables.sh"

# public keys for these seeds have been added to the spec
node1_seed="narrow use math topple stage produce top satoshi rapid satisfy half naive"
node2_seed="lesson online video chapter match pig response inner degree brown often cover"
node3_seed="female city jewel name sausage riot zebra lunch access message buyer gold"
node4_seed="galaxy bundle tuition kite believe opinion page energy wine live farm donkey"
node5_seed="sing earth victory dove tag siege cereal embody scheme grant swear level"

# docker rm -f node1 node2 node3 node4 node5 rpc

for node_name in node1 node2 node3 node4 node5; do
  docker run -d --name $node_name \
    $IMAGE $NODE_BIN \
    --name=$node_name \
    --base-path $VOLUME \
    --chain "${CHAIN_SPEC}" \
    --validator \
    --port $PORT \
    --rpc-cors all \
    --allow-private-ip
  # if there is an error, maybe the container is stopped, try to start it
  if [ $? -ne 0 ]; then
    docker start $node_name
  fi
  docker exec -it $node_name key insert \
    --base-path $VOLUME \
    --chain "${CHAIN_SPEC}" \
    --scheme Sr25519 \
    --suri "$SEED" \
    --key-type aura
  docker exec -it $node_name key insert \
    --base-path $VOLUME \
    --chain "${CHAIN_SPEC}" \
    --scheme Ed25519 \
    --suri "$SEED" \
    --key-type gran
done

docker run -d --name rpc \
  -p ${RPC_PORT}:${RPC_PORT} \
  $IMAGE \
  $NODE_BIN \
  --name rpc \
  --base-path $VOLUME \
  --chain ${CHAIN_SPEC} \
  --rpc-port ${RPC_PORT} \
  --port ${PORT} \
  --rpc-cors all \
  --rpc-external \
  --allow-private-ip \
  --state-pruning archive
