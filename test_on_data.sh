#!/bin/bash

CODER='./target/debug/koda-lzss.exe'
DECODER='python py/decoder.py'

function test_file {
    echo "Testing file $1..."
    local coded_name="$1.lzss"
    local decoded_name="$1"
    local dir="$(dirname "${coded_name}")"
    mkdir -p "test/${dir}"
    echo "  (coding...)"
    ${CODER} -i "$1" -o "test/${coded_name}"
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
