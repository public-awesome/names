MSG=$(cat <<EOF
{
  "associate_address": {
    "name": "$1",
    "address": "$USER"
  }
}
EOF
)

starsd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $USER -y -b block -o json | jq .
 