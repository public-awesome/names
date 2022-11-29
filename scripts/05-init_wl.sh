KEY=$(starsd keys show $USER | jq -r .name)

MSG=$(cat <<EOF
{
  "mint_discount_bps": 5000,  
  "per_address_limit": 50,
  "addresses": ["$ADMIN"]
}
EOF
)

if [ "$ADMIN_MULTISIG" = true ] ; then
  echo 'Using multisig'
  starsd tx wasm instantiate $WHITELIST_CODE_ID "$MSG" --label "WhitelistUpdatable" \
    --no-admin \
    --gas-prices 0.025ustars --gas 50000000 --gas-adjustment 1.9 \
    --from $ADMIN \
    --generate-only > unsignedTx.json

  starsd tx sign unsignedTx.json \
    --multisig=$ADMIN --from $USER --output-document=$KEY.json \
    --chain-id $CHAIN_ID
else
  echo 'Using single signer'
  starsd tx wasm instantiate $WHITELIST_CODE_ID "$MSG" --label "WhitelistUpdatable" \
    --no-admin \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN -y -b block -o json | jq .
fi