#!/bin/bash
set -euo pipefail

# run from project root
cd "$(dirname "$0")/.."

RPC_URL="${RPC_URL:-http://localhost:8547}"
# pre-funded devnode test account from arbitrum docs
PRIVATE_KEY="${PRIVATE_KEY:-0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659}"

echo "=== Building Stylus contract ==="
cargo build --release --target wasm32-unknown-unknown --lib -p scoring-engine

WASM_FILE="target/wasm32-unknown-unknown/release/scoring_engine.wasm"
if [ ! -f "$WASM_FILE" ]; then
  echo "ERROR: WASM file not found at $WASM_FILE"
  exit 1
fi
echo "WASM built: $(wc -c < "$WASM_FILE") bytes"

echo ""
echo "=== Building Solidity contract ==="
cd contracts/scoring-engine-sol
forge build
cd ../..

echo ""
echo "=== Deploying Stylus contract ==="
STYLUS_OUTPUT=$(cargo stylus deploy \
  --endpoint="$RPC_URL" \
  --private-key="$PRIVATE_KEY" 2>&1)

STYLUS_ADDRESS=$(echo "$STYLUS_OUTPUT" | grep "deployed code at address" | grep -o '0x[a-f0-9]*')
if [ -z "$STYLUS_ADDRESS" ]; then
  echo "ERROR: Failed to deploy Stylus contract"
  echo "$STYLUS_OUTPUT"
  exit 1
fi
echo "Stylus contract deployed at: $STYLUS_ADDRESS"

echo ""
echo "=== Deploying Solidity contract ==="
SOL_OUTPUT=$(cd contracts/scoring-engine-sol && forge create \
  src/ScoringEngine.sol:ScoringEngineSol \
  --rpc-url="$RPC_URL" \
  --private-key="$PRIVATE_KEY" \
  --broadcast 2>&1)

SOL_ADDRESS=$(echo "$SOL_OUTPUT" | grep "Deployed to" | awk '{print $3}')
if [ -z "$SOL_ADDRESS" ]; then
  echo "ERROR: Failed to deploy Solidity contract"
  echo "$SOL_OUTPUT"
  exit 1
fi
echo "Solidity contract deployed at: $SOL_ADDRESS"

echo ""
echo "=== Writing .env ==="
cat > .env <<EOF
CONTRACT_ADDRESS=$STYLUS_ADDRESS
SOLIDITY_CONTRACT_ADDRESS=$SOL_ADDRESS
RPC_URL=$RPC_URL
PORT=3001
EOF
echo "Wrote .env with deployed addresses"

echo ""
echo "=== Deployment complete ==="
echo "  Stylus:   $STYLUS_ADDRESS"
echo "  Solidity: $SOL_ADDRESS"
echo ""
echo "Start the API server with: pnpm api"
