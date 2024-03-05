#!/bin/bash

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/variables.sh"

for node in node1 node2 node3 node4 node5 rpc; do
  $EXECUTOR stop $node
  $EXECUTOR rm $node
done

$EXECUTOR rmi chain:latest
$EXECUTOR build -t chain:latest -f scripts/build.Dockerfile .
