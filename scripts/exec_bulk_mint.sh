MSG=$(cat <<EOF
{
  "approve_all": {
    "operator": "$MKT"
  }
}
EOF
)
starsd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $TESTNET_KEY -b block -y -o json | jq .

count=10
for i in {1..count}
do
  name=$(openssl rand -hex 20);
  MSG=$(cat <<EOF
  {
    "mint_and_list": {
      "name": "stargaze-name-$name"
    }
  }
  EOF
  )

  starsd tx wasm execute $MINTER "$MSG" \
  --amount 1000000000ustars \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $TESTNET_KEY -b block -y -o json | jq .
done