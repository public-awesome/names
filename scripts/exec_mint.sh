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
  --from $USER -b block -y -o json | jq .
 

# generate random 20 char string for name
name=$(openssl rand -hex 20);
MSG=$(cat <<EOF
{
  "mint_and_list": {
    "name": "$name"
  }
}
EOF
)

starsd tx wasm execute $MINTER "$MSG" \
  --amount 100ustars \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $USER -b block -y -o json | jq .
 