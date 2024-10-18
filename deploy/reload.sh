#!/usr/bin/bash

set -e

ls
cd ../subparsvc/subparweb
cargo build --release


cd ../../deploy
podman compose up -d

