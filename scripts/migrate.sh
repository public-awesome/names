starsd config node $NODE
starsd config chain-id $CHAIN_ID
starsd config output json
 
starsd tx wasm migrate $1 $MKT_CODE_ID {} \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN -y -b block -o json | jq .

starsd tx wasm migrate $2 $MINTER_CODE_ID {} \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN -y -b block -o json | jq .

starsd tx wasm migrate $3 $COLLECTION_CODE_ID {} \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN -y -b block -o json | jq .

# starsd tx wasm migrate $4 $WHITELIST_CODE_ID {} \
#     --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
#     --from $ADMIN -y -b block -o json | jq .
