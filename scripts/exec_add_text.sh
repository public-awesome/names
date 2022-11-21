MSG=$(cat <<EOF
{
  "update_text_record": {
    "name": "$1",
    "record": {
      "name": "twitter",
      "value": "something"
    }
  }
}
EOF
)

starsd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $USER -y -b block -o json | jq .
 