MSG=$(cat <<EOF
{
  "collection_code_id": $COLLECTION_CODE_ID,
  "marketplace_addr": "$MKT",
  "min_name_length": 3,
  "max_name_length": 63,
  "base_price": "100000000"
}
EOF
)

starsd tx wasm instantiate $MINTER_CODE_ID "$MSG" --label "NameMinter" \
 --admin $ADMIN \
 --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
 --from testnet -y -b block -o json | jq .
 
