#!/bin/sh

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <input_file>"
  exit 1
fi

INPUT="$1"
echo "To work the file must be already sorted"
echo "Checking for duplicate lines in $INPUT"
uniq "$INPUT" > "$INPUT".tmp
mv "$INPUT".tmp "$INPUT"