starsd tx wasm store name_marketplace.wasm --from $TESTNET_KEY \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y

starsd tx wasm store name_minter.wasm --from $TESTNET_KEY \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y

starsd tx wasm store sg721_name.wasm --from $TESTNET_KEY \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y
