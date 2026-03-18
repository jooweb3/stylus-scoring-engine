#!/bin/bash
# run this while cap is recording your terminal
# devnode must already be running: pnpm devnode
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

clear

echo ""
echo "  Stylus Scoring Engine"
echo "  Three-way gas benchmark: Rust vs Stylus vs Solidity"
echo ""
sleep 2

echo "$ pnpm deploy:all"
echo ""
pnpm deploy:all 2>&1 | grep -E "(Building|WASM built|Deploying|deployed at|Deployed to|Writing|Wrote|complete|Stylus:|Solidity:)"
echo ""
sleep 2

echo "$ pnpm api &"
rm -f scoring.db
(cd "$PROJECT_ROOT/api" && cargo run --quiet 2>/dev/null) &
API_PID=$!
sleep 4
echo "  API listening on localhost:3001"
echo ""
sleep 1

echo "$ pnpm benchmark"
echo ""
bash "$PROJECT_ROOT/scripts/benchmark.sh" 2>&1
echo ""
sleep 3

kill $API_PID 2>/dev/null || true
