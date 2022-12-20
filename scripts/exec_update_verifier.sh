MSG=$(cat <<EOF
{
  "update_verifier": {
    "verifier": "$VERIFIER"
  }
}
EOF
)

 if [ "$ADMIN_MULTISIG" = true ] ; then
  echo 'Using multisig'
  KEY=$(starsd keys show $USER | jq -r .name)
  starsd tx wasm execute $COLLECTION "$MSG" \
    --gas-prices 0.025ustars --gas 1000000 --gas-adjustment 1.9 \
    --from $ADMIN \
    --generate-only > unsignedTx.json

  starsd tx sign unsignedTx.json \
    --multisig=$ADMIN --from $USER --output-document=$KEY.json\
    --chain-id $CHAIN_ID
else
starsd tx wasm execute $COLLECTION "$MSG" \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $USER -y -b block -o json | jq .
fi