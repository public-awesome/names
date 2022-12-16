KEY=$(starsd keys show $USER | jq -r .name)

starsd tx sign unsignedTx.json \
    --multisig $ADMIN \
    --from $USER \
    --output-document $KEY.json \
    --chain-id $CHAIN_ID
