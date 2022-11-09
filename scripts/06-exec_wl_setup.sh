KEY=$(starsd keys show $USER | jq -r .name)

MSG=$(cat <<EOF
{
  "update_minter_contract": {
    "minter_contract": "$MINTER"
  }
}
EOF
)

if [ "$ADMIN_MULTISIG" = true ] ; then
  echo 'Using multisig'
  starsd tx wasm execute $WL "$MSG" \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN \
    --generate-only > unsignedTx.json

  starsd tx sign unsignedTx.json \
    --multisig=$ADMIN --from $USER --output-document=$KEY.json \
    --chain-id $CHAIN_ID
else
  echo 'Using single signer'
  starsd tx wasm execute $WL "$MSG" \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN -y -o json | jq .
fi