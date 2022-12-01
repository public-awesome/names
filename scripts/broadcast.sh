for i in {55..68}
do
    starsd tx multisign $i-unsignedTx.json $MULTISIG_NAME s$i.json $i.json > signedTx.json
    starsd tx broadcast signedTx.json
    sleep 66
done
