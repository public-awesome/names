ADDR=stars1ejc9sve7wcvg56acyynz3rn73dtfcg7n49efxpvvragwwy5fu7csskmwr5

# Initialize empty or default values for pagination
last_price="0"
last_token_id=""
last_bidder=""

while : ; do

    MSG=$(cat <<EOF
    {
        "bids_sorted_by_price": {
            "start_after": {
                "price": "$last_price",
                "token_id": "$last_token_id",
                "bidder": "$last_bidder"
            },
            "limit": 100
        }
    })

    # Run the query
    response=$(starsd q wasm contract-state smart $ADDR "$MSG" -o json)

    # Check if the data array is empty
    if [ "$(echo $response | jq '.data | length')" -eq 0 ]; then
        echo "No more results."
        break
    fi

    # Print the response (or handle it as needed)
    echo "$response"

    # Extract last_price, last_token_id, and last_bidder from the last element of the data array
    last_price=$(echo $response | jq -r '.data[-1].amount')
    last_token_id=$(echo $response | jq -r '.data[-1].token_id')
    last_bidder=$(echo $response | jq -r '.data[-1].bidder')

    # Add a sleep if necessary to prevent rate-limiting issues
    sleep 1
done
