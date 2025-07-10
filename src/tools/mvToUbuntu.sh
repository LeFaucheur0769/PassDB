#!/bin/sh

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <input_dir> <output_dir>"
  exit 1
fi

INPUT="$1"
OUTPUT="$2"

echo "Sorting files from '$INPUT' into '$OUTPUT'..."

# Create the output directory if it doesn't exist
mkdir -p "$OUTPUT"

# Traverse all files in the input directory
rsync -av --append --progress --remove-source-files "$INPUT" "$OUTPUT"
done
