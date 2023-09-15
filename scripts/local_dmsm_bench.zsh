#!/usr/bin/env bash
set -ex
trap "exit" INT TERM
trap "kill 0" EXIT

cargo build --example local_dmsm_bench
BIN=../target/debug/examples/local_dmsm_bench
RUST_BACKTRACE=0 RUST_LOG=msm $BIN

# Check for macOS and use `open` command, otherwise use `feh`
if [[ "$OSTYPE" == "darwin"* ]]; then
    open ./msm_benchmark.png
else
    feh ./msm_benchmark.png
fi
