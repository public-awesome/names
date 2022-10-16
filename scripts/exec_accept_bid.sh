MSG=$(cat <<EOF
{
  "approve": {
    "spender": "$MKT",
    "token_id": "$1"
  }
}
EOF
)

starsd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from testnet -b block -y -o json | jq .
 

MSG=$(cat <<EOF
{
  "accept_bid": {
    "token_id": "$1",
    "bidder": "$2"
  }
}
EOF
)

starsd tx wasm execute $MKT "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from testnet -b block -y -o json | jq .
 