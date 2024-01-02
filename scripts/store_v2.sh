starsd config node $NODE
starsd config chain-id $CHAIN_ID
starsd config output json

STORE

starsd tx wasm store artifacts/name_marketplace.wasm --from hot-wallet \
    --keyring-backend test \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

starsd tx wasm store artifacts/sg721_name.wasm --from hot-wallet \
    --keyring-backend test \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

 