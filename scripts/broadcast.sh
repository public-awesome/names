starsd tx multisign unsignedTx.json $MULTISIG_NAME $1 $2 $3 > signedTx.json

starsd tx broadcast signedTx.json