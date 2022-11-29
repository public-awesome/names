MSG=$(cat <<EOF
{
  "approve_all": {
    "operator": "$MKT"
  }
}
EOF
)

starsd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ustars --gas 50000000 --gas-adjustment 1.9 \
  --from $ADMIN \
  --generate-only > unsignedTx-approve.json
 

MSG=$(cat <<EOF
{
  "mint_and_list": {
    "name": "$1"
  }
}
EOF
)

starsd tx wasm execute $MINTER "$MSG" \
  --amount 50000000ustars \
  --gas-prices 0.025ustars --gas 50000000 --gas-adjustment 1.9 \
  --from $ADMIN \
  --generate-only > unsignedTx.json
 