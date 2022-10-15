MSG=$(cat <<EOF
{
  "collection": {}
}
EOF
)

starsd q wasm contract-state smart $MINTER "$MSG"
 

MSG=$(cat <<EOF
{
  "params": {}
}
EOF
)

starsd q wasm contract-state smart $MINTER "$MSG"
 