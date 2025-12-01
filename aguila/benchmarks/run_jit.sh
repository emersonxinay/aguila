#!/bin/bash
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}=== Benchmarking JIT (VM Mode) ===${NC}"
cargo build --release
AGUILA=target/release/aguila

echo -e "\n>>> Running fib.ag in VM mode (JIT enabled)..."
time $AGUILA vm benchmarks/fib.ag > benchmarks/output.txt 2>&1
