#!/bin/bash

examples_dir="$(dirname "$(realpath "$0")")"
samples_dir="${examples_dir}/samples"

mkdir -p "$samples_dir"

output_file="$samples_dir/100mb.data"

fallocate -l 100M "$output_file"

echo "100 MB file created at: .${output_file/$(pwd)/}"
