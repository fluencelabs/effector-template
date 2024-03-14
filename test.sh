#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# build the effector module
cd effector
marine build --release

WASM_LOG=debug cargo nextest run --release --no-fail-fast --nocapture
