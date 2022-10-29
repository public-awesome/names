MSG=$(cat <<EOF
{
  "config": {}
}
EOF
)

starsd q wasm contract-state smart $WL "$MSG"
 

MSG=$(cat <<EOF
{
  "per_address_limit": {}
}
EOF
)

starsd q wasm contract-state smart $WL "$MSG"


MSG=$(cat <<EOF
{
  "is_processable": {
    "address": "$ADMIN"
  }
}
EOF
)

starsd q wasm contract-state smart $WL "$MSG"