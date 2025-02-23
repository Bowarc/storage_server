#!/bin/bash

set -e

RUST_LOG="walrus=error"

# Idk if it's really the goal of this script
# # Make sure rustup has the target, only procduces a dbg message if already installed
# rustup target add wasm32-unknown-unknown

if [ "$1" = release ] || [ "$1" = r ]
then
  echo Building front using release mode
  cargo build -p front --release --target=wasm32-unknown-unknown
  mode=release
else
  echo Building front using debug mode
  cargo build -p front --target=wasm32-unknown-unknown
  mode=debug
fi

echo Bindgen
wasm-bindgen --target=web --out-dir=./target/wasm-bindgen/$mode ./target/wasm32-unknown-unknown/$mode/front.wasm --no-typescript

if ! [ -d "./static/" ]; then
  echo Creating ouput directory
  mkdir ./static/
fi

cp ./target/wasm-bindgen/$mode/* ./static/

echo Done
