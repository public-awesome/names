MSG=$(cat <<EOF
{
  "minter": {}
}
EOF
)

starsd q wasm contract-state smart $COLLECTION "$MSG"
 
