#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

RPC_URL="${RPC_URL:-http://localhost:8547}"
# pre-funded devnode test account from arbitrum docs
PRIVATE_KEY="${PRIVATE_KEY:-0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659}"

ACCOUNTS=(
  "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
  "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
  "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
  "0x90F79bf6EB2c4f870365E785982E1f101E93b906"
  "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65"
)

echo "Funding test accounts..."
for ACCOUNT in "${ACCOUNTS[@]}"; do
  echo "Funding $ACCOUNT..."
  cast send --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" \
    --value "1 ether" "$ACCOUNT" > /dev/null 2>&1
  BALANCE=$(cast balance --rpc-url "$RPC_URL" "$ACCOUNT")
  echo "  Balance: $BALANCE"
done

echo "Done."
