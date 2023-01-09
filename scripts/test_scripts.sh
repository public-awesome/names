# test all contract functionality in this script
# USER, BIDDER, USER2 should be different addresses in .env

# pause and unpause mint
echo "pause mint";
./exec_pause.sh true
echo "unpause mint";
./exec_pause.sh false

# mint a new token
name=$(openssl rand -hex 20);
echo "mint new token $name";
./exec_mint.sh $name

# add text record
echo "add text record";
./exec_add_text.sh $name

# associate address
echo "associate address";
./exec_assoc.sh $name

# reverse look up
echo "reverse look up";
./query_lookup.sh $name

# token info look up
echo "token info look up";
./query_token_info.sh $name

# metadata look up
echo "metadata look up";
./query_metadata.sh $name

# make a bid
echo "make a bid";
./exec_bid.sh $name

# accept bid
echo "accept bid";
bidder_addr=$(starsd keys show $BIDDER | jq -r '.address')
./exec_accept_bid.sh $name $bidder_addr

# move public time to the future
echo "move public time to the future";
TIME=$(date -v+1000S +%s)
./exec_update_public_time.sh "$(echo $TIME)000000000"

sleep 1

# mint a new token even though not on whitelist and public mint hasn't started yet
name=$(openssl rand -hex 20);
echo "mint a new token even though not on whitelist and public mint has not started yet $name";
./exec_mint_specific_user.sh $name $BIDDER 100000000

# make new whitelist
echo "make new whitelist";
WL2=$(bash 05-init_wl.sh | jq -r '.logs[0].events[0].attributes[0].value')

# add addresses to whitelist
echo "add address to whitelist";
./exec_wl_add_addrs.sh "[\"$BIDDER\"]"



# add wl to minter
echo "add wl to minter";
sleep 5
./06-exec_minter_add_wl.sh $WL2



# wl mint
sleep 5
echo "wl mint";
name=$(openssl rand -hex 20);
sleep 60
./exec_mint_specific_user.sh $name $BIDDER 50000000

# update public time
sleep 5
echo "update public time";
TIME=$(date -v+1000S +%s)
./exec_update_public_time.sh "$(echo $TIME)000000000"

# test public mint and whitelist mint
sleep 5
echo "test public mint and whitelist mint";
name=$(openssl rand -hex 20);
./exec_mint_specific_user.sh $name $USER2 100000000

# sleep past rate limit and mint
sleep 60
name=$(openssl rand -hex 20);
./exec_mint_specific_user.sh $name $BIDDER 50000000