MSG=$(cat <<EOF
{
  "migrate_bids": {
    "limit": 100
  }
}
EOF
)

starsd tx wasm execute stars1ejc9sve7wcvg56acyynz3rn73dtfcg7n49efxpvvragwwy5fu7csskmwr5 "$MSG" \
  --from hot-wallet --keyring-backend test \
  --amount 6000000ustars \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  -b block -o json | jq .