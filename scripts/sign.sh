starsd tx sign $1-unsignedTx.json \
    --multisig=$FOUNDATION \
    --from shane1 \
    --output-document=s$1.json \
    --chain-id $CHAIN_ID \
    --sign-mode amino-json \
    --sequence $1 \
    --offline \
    --account-number 123
