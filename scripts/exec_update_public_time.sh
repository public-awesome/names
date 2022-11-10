# ./exec_update_public_time.sh 1668116209000000000

MSG=$(cat <<EOF
{
  "update_config": {
    "config": {
        "public_mint_start_time": "$1"
    }
  }
}
EOF
)

if [ "$ADMIN_MULTISIG" = true ] ; then
  echo 'Using multisig'
  starsd tx wasm execute $MINTER "$MSG" \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN \
    --generate-only > unsignedTx.json

  starsd tx sign unsignedTx.json \
    --multisig=$ADMIN --from $USER --output-document=$USER.json \
    --chain-id $CHAIN_ID
else
  echo 'Using single signer'
  starsd tx wasm execute $MINTER "$MSG" \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN -y -b block -o json | jq .
fi