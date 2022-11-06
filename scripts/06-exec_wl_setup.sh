MSG=$(cat <<EOF
{
  "update_minter_contract": {
    "minter_contract": "$MINTER"
  }
}
EOF
)

starsd tx wasm execute $WL "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $ADMIN -y -o json | jq .