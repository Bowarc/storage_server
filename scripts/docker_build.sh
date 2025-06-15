#/bin/bash

set -e

echo Building image
docker build -t storage_server:latest .
