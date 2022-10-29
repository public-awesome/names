MSG=$(cat <<EOF
{
  "setup": {
    "minter": "$MINTER",
    "collection": "$COLLECTION"
  }
}
EOF
)

starsd tx wasm execute $MKT "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $TESTNET_KEY -y -o json | jq .
 
