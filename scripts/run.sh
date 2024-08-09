#!/bin/sh
set -e

mode=debug # debug, release

echo Running server
if [ "$mode" = release ]
then
  cargo run -p back --release
else
  cargo run -p back
fi