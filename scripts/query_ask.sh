MSG=$(cat <<EOF
{
  "ask": {
    "token_id": "f0bb4793a8516b9ec5767548c4b7db503c4a67f8"
  }
}
EOF
)

starsd q wasm contract-state smart stars1ejc9sve7wcvg56acyynz3rn73dtfcg7n49efxpvvragwwy5fu7csskmwr5 "$MSG"
 
