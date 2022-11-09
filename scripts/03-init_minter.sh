KEY=$(starsd keys show $USER | jq -r .name)

MSG=$(cat <<EOF
{
  "collection_code_id": $COLLECTION_CODE_ID,
  "admin": "$ADMIN",
  "marketplace_addr": "$MKT",
  "min_name_length": 3,
  "max_name_length": 63,
  "base_price": "100000000",
  "fair_burn_bps": 5000,
  "whitelists": [],
  "verifier": "$VERIFIER"
}
EOF
)

if [ "$ADMIN_MULTISIG" = true ] ; then
  echo 'Using multisig'
  starsd tx wasm instantiate $MINTER_CODE_ID "$MSG" --label "NameMinter" \
    --admin $ADMIN \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN \
    --generate-only > unsignedTx.json

  starsd tx sign unsignedTx.json \
    --multisig=$ADMIN --from $USER --output-document=$KEY.json \
    --chain-id $CHAIN_ID
else
  echo 'Using single signer'
  starsd tx wasm instantiate $MINTER_CODE_ID "$MSG" --label "NameMinter" \
    --admin $ADMIN \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN -y -b block -o json | jq .
fi