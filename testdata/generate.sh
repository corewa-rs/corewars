#!/bin/bash

if [[ $# -lt 2 ]]; then
    echo "Usage: generate.sh PMARS REDCODE [REDCODE ...]"
    echo
    echo "    PMARS is the path to the pMars executable"
    echo
    echo "    REDCODE is a path to input testdata"

    exit 1
fi

PMARS=$(realpath "$1")
shift

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

for file in "$@"; do
    OUT_FILE="${file/input/expected_output}"
    "$PMARS" -r 0 "$file" | tail -n +3 > "$OUT_FILE"
    "$SCRIPT_DIR/normalize.py" "$OUT_FILE"
done
