MSG=$(cat <<EOF
{
  "asks_by_renew_time": {
    "max_time": "1704227594000000000",
    "limit": 100
  }
}
EOF
)

starsd q wasm contract-state smart stars1ejc9sve7wcvg56acyynz3rn73dtfcg7n49efxpvvragwwy5fu7csskmwr5 "$MSG"
 
