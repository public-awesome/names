MSG=$(cat <<EOF
{
  "trading_fee_bps": 200,
  "min_price": "5000000"
}
EOF
)

starsd tx wasm instantiate $MKT_CODE_ID "$MSG" --label "NameMarketplace" \
 --admin $ADMIN \
 --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
 --from $TESTNET_KEY -y -b block -o json | jq .
 
