MSG=$(cat <<EOF
{
  "set_bid": {
    "token_id": "$1"
  }
}
EOF
)

starsd tx wasm execute $MKT "$MSG" \
  --amount $2000000ustars \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from testnet -y -b block -o json | jq .
 
