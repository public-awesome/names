MSG=$(cat <<EOF
{
  "max_renewals_per_block": 100,
  "valid_bid_query_limit": 100,
  "renew_window": 2592000,
  "renewal_bid_percentage": "0.005",
  "operator": "stars19mmkdpvem2xvrddt8nukf5kfpjwfslrsu7ugt5"
}
EOF
)

starsd tx wasm migrate stars1ejc9sve7wcvg56acyynz3rn73dtfcg7n49efxpvvragwwy5fu7csskmwr5 3500 "$MSG" \
        --from hot-wallet --keyring-backend test \
        --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
        -b block -o json | jq .