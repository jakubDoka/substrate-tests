#!/bin/bash
#
# By sourcing this file, these variables
# can be reused on scripts, and also on docker build.
#
# Most of the variables follow the flag names
# described in the node binary flags, except for VOLUME,
# which in the node is PATH.

# absolute path of the script dir
export SCRIPTS_DIR="$(dirname "$(readlink -f "$0")")"
export IMAGE="chain:latest"
export CHAIN="local"
# dir of the persistent volume of the node container,
# chain spec and state will be in this dir.
# base_path
# default base_path is $HOME/.local/share/<CHAIN_NAME>
export VOLUME="/data"
export CHAIN_SPEC="${VOLUME}/${CHAIN}_spec.json"
# where the node binary is located on the node container
export NODE_BIN="/opt/node-template"
# p2p port
export PORT=30333
# RPC port
export RPC_PORT=9944
