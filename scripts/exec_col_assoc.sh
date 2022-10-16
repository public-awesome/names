MSG=$(cat <<EOF
{
  "associate_address": {
    "name": "$1",
    "address": "$2",
  }
}
EOF
)

starsd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from testnet -y -b block -o json | jq .
 