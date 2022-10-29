# ./exec_wl_add_addrs.sh '["stars1wh3wjjgprxeww4cgqyaw8k75uslzh3sd3s2yfk"]'
MSG=$(cat <<EOF
{
  "add_addresses": {
    "addresses": $1
  }
}
EOF
)

starsd tx wasm execute $WL "$MSG" \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
  --from $TESTNET_KEY -b block -y -o json | jq .
 