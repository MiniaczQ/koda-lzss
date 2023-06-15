#!/bin/bash

DISTANCE_BIT_WIDTH=${1:-8}
LENGTH_BIT_WIDTH=${2:-8}

CODER="./target/debug/koda-lzss.exe -d ${DISTANCE_BIT_WIDTH} -m ${LENGTH_BIT_WIDTH}"
DECODER="python py/decoder.py --window-size $((2 ** ${DISTANCE_BIT_WIDTH})) --length-width ${LENGTH_BIT_WIDTH}"

function test_file {
    echo "Testing file $1..."
    local coded_name="$1.lzss"
    local decoded_name="$1"
    local dir="$(dirname "${coded_name}")"
    mkdir -p "test/${dir}"
    echo "  (coding...)"
    ${CODER} "$1" "test/${coded_name}"
    echo "  (decoding...)"
    ${DECODER} "test/${coded_name}" "test/${decoded_name}"
    echo "  (diffing...)"
    diff -sq "$1" "test/${decoded_name}"
    echo ""
}
export -f test_file
export CODER
export DECODER

find ./data -name \*.pgm -exec bash -c 'test_file "$0"' {} \;
