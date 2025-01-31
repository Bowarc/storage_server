#!/bin/bash

set -e

RUST_LOG="walrus=error"

mode=debug # debug, release

out_dir=./front/out

echo Compiling front
if [ "$mode" = release ]
then
  cargo build -p front --release --target=wasm32-unknown-unknown
else
  cargo build -p front --target=wasm32-unknown-unknown
fi

echo Bindgen
wasm-bindgen --target=web --out-dir=./target/wasm-bindgen/$mode ./target/wasm32-unknown-unknown/$mode/front.wasm --no-typescript

if ! [ -d $out_dir ]; then
    echo Creating ouput directory
    mkdir $out_dir
else
    echo Updating ouput directory
fi

cp ./target/wasm-bindgen/$mode/* $out_dir

if ! [ -d "./static/" ]; then
    echo Creating ouput directory
    mkdir ./static/
fi

cp $out_dir/* ./static/

echo Done
