#!/bin/bash

DISTANCE_BIT_WIDTH=${1:-8}
LENGTH_BIT_WIDTH=${2:-8}

echo
echo "[Running tests for code words with ${DISTANCE_BIT_WIDTH}-bit distance and ${LENGTH_BIT_WIDTH}-bit length]"
echo

CODER="./target/release/encoder.exe -d ${DISTANCE_BIT_WIDTH} -m ${LENGTH_BIT_WIDTH}"
DECODER="python decoder/decoder.py --window-size $((2 ** ${DISTANCE_BIT_WIDTH})) --length-width ${LENGTH_BIT_WIDTH}"

function test_file {
    echo "Testing file $1..."
    local coded_name="$1.lzss"
    local decoded_name="$1"
    local dir="$(dirname "${coded_name}")"
    mkdir -p "test/${dir}"
    echo "  (coding...)"
    { time ${CODER} "$1" "test/${coded_name}" ;} 2>&1
    echo "  (decoding...)"
    { time ${DECODER} "test/${coded_name}" "test/${decoded_name}" ;} 2>&1
    echo "  (diffing...)"
    diff -sq "$1" "test/${decoded_name}"
    echo ""
}
export -f test_file
export CODER
export DECODER

find ./data -name \*.pgm -exec bash -c 'test_file "$0"' {} \;
