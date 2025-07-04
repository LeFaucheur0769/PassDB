#!/bin/sh

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <input_file> <output_directory>"
  exit 1
fi

INPUT="$1"
OUTPUT_DIR="$2"
OUTPUT_FILE="$OUTPUT_DIR/sorted.txt"

mkdir -p "$OUTPUT_DIR"  
echo "Sorting $INPUT into $OUTPUT_FILE"

awk -F: '{if (NF >= 3) print $2 ":" $3 ":" $1}' "$INPUT" > "$OUTPUT_FILE"
sed -i 's/\r//' "$OUTPUT_FILE"