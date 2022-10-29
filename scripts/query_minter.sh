MSG=$(cat <<EOF
{
  "collection": {}
}
EOF
)

starsd q wasm contract-state smart $MINTER "$MSG"
 

MSG=$(cat <<EOF
{
  "admin": {}
}
EOF
)

starsd q wasm contract-state smart $MINTER "$MSG"


MSG=$(cat <<EOF
{
  "whitelists": {}
}
EOF
)

starsd q wasm contract-state smart $MINTER "$MSG"