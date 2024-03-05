#!/bin/bash
#
# Spawn all the node containers.
# To update the chain spec of the nodes, after running this script: `./update_spec.sh`

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/variables.sh"

# public keys for these seeds have been added to the spec
# alice 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
node1_seed="narrow use math topple stage produce top satoshi rapid satisfy half naive"
node2_seed="lesson online video chapter match pig response inner degree brown often cover"
node3_seed="female city jewel name sausage riot zebra lunch access message buyer gold"
node4_seed="galaxy bundle tuition kite believe opinion page energy wine live farm donkey"
node5_seed="sing earth victory dove tag siege cereal embody scheme grant swear level"

for node_name in node1 node2 node3 node4 node5; do
  seed_name="${node_name}_seed"
  SURI=${!seed_name}

  $EXECUTOR run -d \
    --name $node_name \
    -e "VALIDATOR=TRUE" \
    -e "NAME=$node_name" \
    -e "SEED=$SURI" \
    $IMAGE "${VOLUME}/docker_start.sh" \
  # # if there is an error, maybe the container is stopped, try to start it
  if [ $? -ne 0 ]; then
    EXECUTOR start $node_name
  fi
done

$EXECUTOR run -d \
  --name rpc \
  -p ${RPC_PORT}:${RPC_PORT} \
  -e "NAME=rpc" \
  -e "SEED=$SURI" \
  $IMAGE "${VOLUME}/docker_start.sh" \
