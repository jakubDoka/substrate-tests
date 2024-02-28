#!/bin/bash
#
# Update the chain spec of all node containers on their volume partitions.
# The nodes must already be running before executing this script: `./test_network.sh`

SCRIPTS_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPTS_DIR/variables.sh"
tmp="${SCRIPTS_DIR}/.tmp"
spec_tmp="${tmp}/${CHAIN}_spec.json"
mkdir -p $tmp

# generate the build-spec for the current chain
target/release/node-template build-spec --chain=$CHAIN --disable-default-bootnode > $spec_tmp

for node in node1 node2 node3 node4 node5 rpc; do
  # try to copy the build-spec file and move to the volume of the current node
  # the container does not need to be running, (on `docker ps`)
  # but it needs to exist (previously run with `docker run --name $node`.
  docker cp $spec_tmp $node_name:$CHAIN_SPEC

  if [ $? -ne 0 ]; then
    echo -e "⚠️ Error!\n"
    echo -e "The container '${node_name}' does not exist. Make sure you used 'docker run --name $node_name' or use the script './scripts/test_network.sh' to spawn the node containers.\n"
    echo -e "After that, try again."
  fi
done
