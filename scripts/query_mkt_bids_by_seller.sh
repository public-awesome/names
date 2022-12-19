MSG=$(cat <<EOF
{
  "bids_for_seller": { "seller": "$1" }
}
EOF
)

starsd q wasm contract-state smart $MKT "$MSG"
 
