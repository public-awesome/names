MSG=$(cat <<EOF
{
  "setup": {
    "minter": "$MINTER",
    "collection": "$COLLECTION"
  }
}
EOF
)

# Any account can setup the name marketplace contract 
starsd tx wasm execute $MKT "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $USER -y -o json | jq .
