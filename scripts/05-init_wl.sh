MSG=$(cat <<EOF
{
  "mint_discount_bps": 5000,  
  "per_address_limit": 1,
  "addresses": []
}
EOF
)

starsd tx wasm instantiate $WHITELIST_CODE_ID "$MSG" --label "WhitelistUpdatable" \
 --admin $ADMIN \
 --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
 --from $TESTNET_KEY -y -b block -o json | jq .