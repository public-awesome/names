MSG=$(cat <<EOF
{
  "update_metadata": {
    "name": "$1",
    "metadata": $2

  }
}
EOF
)

starsd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $USER -y -b block -o json | jq .
 