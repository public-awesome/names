starsd config node $NODE
starsd config chain-id $CHAIN_ID
starsd config output json

starsd tx wasm store name_marketplace.wasm --from $GOV \
    --instantiate-only-address $ADMIN \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

starsd tx wasm store name_minter.wasm --from $GOV \
    --instantiate-only-address $ADMIN \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

starsd tx wasm store sg721_name.wasm --from $GOV \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

starsd tx wasm store whitelist_updatable.wasm --from $GOV \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id
