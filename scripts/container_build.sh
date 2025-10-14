#/bin/bash

set -e

echo Building image
podman build -t storage_server:latest .
