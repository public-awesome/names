# test all contract functionality in this script

# pause and unpause mint
./exec_pause.sh true
./exec_pause.sh false

# mint a new token
name=$(openssl rand -hex 20);
./exec_mint.sh $name

# update metadata
metadata=$(cat <<EOF
{
    "records": [{
        "name": "discord",
        "value": "reallycool"
    }]
}
EOF
)
./exec_update_metadata.sh $name $metadata

# add text record
record=$(cat <<EOF
{
    "name": "twitter",
    "value": "something"
}
EOF
)

./exec_add_text.sh $name $record

# associate address
./exec_assoc.sh $name

# reverse look up
./query_lookup.sh $name

# make a bid
./exec_bid.sh $name


# accept bid
bidder=$(starsd keys show $BIDDER | jq -r '.address')
./exec_accept_bid.sh $name $bidder

# make new whitelist