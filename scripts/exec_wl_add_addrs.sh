MSG=$(cat <<EOF
{
  "add_addresses": {
    "addresses": $1
  }
}
EOF
)

starsd tx wasm execute $WL2 "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $TESTNET_KEY -b block -y -o json | jq .
 