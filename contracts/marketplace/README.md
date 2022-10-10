# Name Marketplace

Name Marketplace is mostly used by [name-minter](../name-minter/) to mint and list names. When a name is minted, an ask order is created on this marketplace. Asks remain in perpetuity.

### Authorizing Marketplace

In order to place a bid or set an ask, the owner needs to grant approval to the marketplace contract for transferring the NFT. This can be done with `ApproveAll` for all NFTs in the collection. This needed since the `token_id` is not known before minting a name.
