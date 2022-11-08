MSG=$(cat <<EOF
{
  "name": { "address": "$USER" }
}
EOF
)

starsd q wasm contract-state smart $COLLECTION "$MSG"
 
