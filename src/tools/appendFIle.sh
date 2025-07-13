#!/bin/sh

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <input_dir> <output_dir>"
  exit 1
fi
src_dir="$1"
dst_dir="$2"
find "$src_dir" -type f | while read -r src_file; do
  # Remove the exact source dir + trailing slash from the full source file path to get relative path inside 'output'
  rel_path="${src_file#$src_dir/}"

  dst_file="$dst_dir/$rel_path"

  mkdir -p "$(dirname "$dst_file")"

  cat "$src_file" >>"$dst_file"

  #rm -f "$src_file"
done
