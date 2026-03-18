#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

API_URL="${API_URL:-http://localhost:3001}"

echo "=== Adding scoring rules ==="
for i in $(seq 1 10); do
  WEIGHT=$((i * 5))
  THRESHOLD=$((i * 1000))
  BONUS=$((60 + i * 3))
  curl -s -X POST "$API_URL/rules" \
    -H 'Content-Type: application/json' \
    -d "{\"factor_name\":\"factor_${i}\",\"weight\":${WEIGHT},\"threshold\":${THRESHOLD},\"direction\":\"Above\",\"penalty_or_bonus\":${BONUS}}" > /dev/null
done
echo "10 rules added"

echo ""
echo "=== Running three-way benchmark ==="
echo "  10 factors, 10 correlation pairs, 50 iterations"
echo ""

RESULT=$(curl -s -X POST "$API_URL/benchmark" \
  -H 'Content-Type: application/json' \
  -d '{
  "entity": {
    "factors": {
      "factor_1": 50000, "factor_2": 42000, "factor_3": 38000,
      "factor_4": 75000, "factor_5": 12000, "factor_6": 91000,
      "factor_7": 33000, "factor_8": 67000, "factor_9": 28000,
      "factor_10": 55000
    }
  },
  "correlations": [
    {"factor_a": "factor_1", "factor_b": "factor_2", "weight": 15},
    {"factor_a": "factor_1", "factor_b": "factor_5", "weight": 25},
    {"factor_a": "factor_2", "factor_b": "factor_3", "weight": 10},
    {"factor_a": "factor_3", "factor_b": "factor_7", "weight": 20},
    {"factor_a": "factor_4", "factor_b": "factor_6", "weight": 30},
    {"factor_a": "factor_5", "factor_b": "factor_8", "weight": 12},
    {"factor_a": "factor_6", "factor_b": "factor_9", "weight": 18},
    {"factor_a": "factor_7", "factor_b": "factor_10", "weight": 22},
    {"factor_a": "factor_8", "factor_b": "factor_1", "weight": 14},
    {"factor_a": "factor_9", "factor_b": "factor_4", "weight": 28}
  ],
  "iterations": 50
}')

echo "$RESULT" | python3 -c "
import sys,json
d=json.load(sys.stdin)
print(f\"{'Label':20s} {'Score':>6} {'Gas':>12} {'Savings':>10}\")
print('=' * 52)
sol_gas = None
for b in d['benchmarks']:
    gas = b['gas_used']
    gas_str = str(gas) if gas else 'n/a'
    savings = ''
    if b['label'] == 'onchain_solidity' and gas:
        sol_gas = gas
    if b['label'] == 'onchain_stylus' and gas and sol_gas:
        pct = ((sol_gas - gas) / sol_gas) * 100
        savings = f'{pct:.1f}% cheaper'
    elif b['label'] == 'onchain_solidity' and gas:
        sol_gas = gas
        # recalc if stylus was first
        for prev in d['benchmarks']:
            if prev['label'] == 'onchain_stylus' and prev['gas_used']:
                pct = ((gas - prev['gas_used']) / gas) * 100
                pass
    print(f\"{b['label']:20s} {b['score']:>6} {gas_str:>12} {savings:>10}\")

# summary
for b in d['benchmarks']:
    if b['label'] == 'onchain_stylus':
        stylus_gas = b['gas_used']
    if b['label'] == 'onchain_solidity':
        sol_gas = b['gas_used']
if stylus_gas and sol_gas:
    pct = ((sol_gas - stylus_gas) / sol_gas) * 100
    print(f\"\nStylus saves {pct:.1f}% gas vs Solidity on this workload\")
"
