MSG=$(cat <<EOF
{
  "add_whitelist": {
    "address": "$WL"
  }
}
EOF
)

starsd tx wasm execute $MINTER "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $ADMIN -y -o json | jq .