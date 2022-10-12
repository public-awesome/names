MSG=$(cat <<EOF
{
  "config": {}
}
EOF
)

starsd q wasm contract-state smart $MKT "$MSG"
 
