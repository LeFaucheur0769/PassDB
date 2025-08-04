#!/bin/sh

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <input_file>"
  exit 1
fi

INPUT="$1"
sort -o "$INPUT" -u "$INPUT"