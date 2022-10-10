# Name Marketplace

Name Marketplace is mostly used by [name-minter](../name-minter/) to mint and list names. When a name is minted, an ask order is created on this marketplace. Asks remain in perpetuity.

## WARNING: NOT FOR COMMERCIAL USE

This repo is under a business source license simliar to Uniswap V3. This means it is **not available** under an open source license for a period of time. Please see [LICENSE](LICENSE) for full details.

### Authorizing Marketplace

In order to place a bid or set an ask, the owner needs to grant approval to the marketplace contract for transferring the NFT. This can be done with `ApproveAll` for all NFTs in the collection. This needed since the `token_id` is not known before minting a name.

## DISCLAIMER

STARGAZE NAME MARKETPLACE IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating or instantiating Stargaze smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of Stargaze, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value. Although Public Awesome, LLC and it's affilliates developed the initial code for Stargaze, it does not own or control the Stargaze network, which is run by a decentralized validator set.
