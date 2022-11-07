starsd tx multisign unsignedTx.json $MULTISIG_NAME $FILE1 $FILE2 $FILE3 > signedTx.json

starsd tx broadcast signedTx.json