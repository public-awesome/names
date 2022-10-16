MSG=$(cat <<EOF
{
  "nft_info": { "token_id": "$1" }
}
EOF
)

starsd q wasm contract-state smart $COLLECTION "$MSG"
 
