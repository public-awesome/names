# test all contract functionality in this script

# pause and unpause mint

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