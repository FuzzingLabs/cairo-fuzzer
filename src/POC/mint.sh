#!/bin/bash

set -eu

address=$1
port=$2

curl localhost:$2/mint \
    -H "Content-Type: application/json" \
    -d "{ \"address\": \"$address\", \"amount\": 1000000000000000000, \"lite\": false }"
