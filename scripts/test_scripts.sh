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
WL2=$(bash 05-init_wl.sh | jq -r '.logs[0].events[0].attributes[0].value')

# add addresses to whitelist
./exec_wl_add_addrs.sh '["stars1u5kav800kkkrzyvad67zhdmn4xajg6t5j7jm7k"]'

# update public time
TIME=$(date -v+1M +%s)
./exec_update_public_time.sh "$(echo $TIME)000000000"
