MSG=$(cat <<EOF
{
  "name": { "address": "$ADMIN" }
}
EOF
)

starsd q wasm contract-state smart $COLLECTION "$MSG"
 
