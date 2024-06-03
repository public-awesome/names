MSG=$(cat <<EOF
{
  "params": {}
}
EOF
)

starsd q wasm contract-state smart stars1ejc9sve7wcvg56acyynz3rn73dtfcg7n49efxpvvragwwy5fu7csskmwr5 "$MSG"
 
