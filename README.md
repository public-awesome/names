# Stargaze Names: A Cosmos IBC Name Service

## API Docs

See [API Docs](./API.md)

## Architecture

Names are stored without the TLD so they can be mapped to _any_ Cosmos address. All names can be resolved to an address that is derived via the same Cosmos key derivation path (118).

When you buy a Stargaze Name, you are really getting a name on _every_ Cosmos chain. Any chain can lookup a name by its local address over IBC. Similarly, any chain can mint a name over IBC that resolves to a local address.

```
bobo -> D93385094E906D7DA4EBFDEC2C4B167D5CAA431A (in hex)
```

Now this can be resolved per chain:

```
bobo.stars  -> stars1myec2z2wjpkhmf8tlhkzcjck04w25sc6ymhplz
bobo.cosmos -> cosmos1myec2z2wjpkhmf8tlhkzcjck04w25sc6y2xq2r
```

This architecture enables Stargaze Names to be a truly Interchain name service since it can mint and resolve names for any Cosmos chain.

Chains that use different account types or key derivation paths can have support added later by migrating contracts. Name contracts are community-owned contracts that can be migrated by Stargaze governance.

### Annual Auction

- [x] When a name is minted it is automatically listed in Name Marketplace
- [x] Owner can accept the top bid at any time
- [ ] After 1 year, owner has to pay 0.5% of the top bid to keep the name
- [ ] If a bid is placed for 4 weeks, name value rises to this value
- [ ] If fee is not paid, name is transferred to the bidder
- [ ] If there are no bids, there is a minimum fee to keep the name based on the number of characters
- [ ] Cap annual fee at X per year

## Initial Fees

```
5+ chars = 100 STARS
4 chars  = 1,000 STARS
3 chars  = 10,000 STARS
```

## Contracts

### [SG-721 Name](./contracts/sg721-name/README.md)

A cw721 contract with on-chain metadata for a name.

Types used in metadata:

```rs
pub struct TextRecord {
    pub name: String,           // "twitter"
    pub value: String,          // "shan3v"
    pub verified: Option<bool>  // verified by oracle
}
```

```rs
pub struct Metadata {
    pub image_nft: Option<NFT>,
    pub record: Vec<TextRecord>,
}
```

Names are designed to be as flexible as possible, allowing generic `TextRecord` types to be added. Each record has a `verified` field that can only be modified by a verification oracle. For example, a Twitter verification oracle can verify a user's signature in a tweet, and set `verified` to `true`. Text records can also be used to link the name to other name services such as ENS.

`profile_nft` points to another NFT with on-chain metadata for profile information such as bio, header (banner) image, and follower information. This will be implemented as a separate collection.

### [Name Minter](./contracts/name-minter/README.md)

Name minter is responsible for minting, validating, and updating names and their metadata.

### [Name Marketplace](./contracts/marketplace/README.md)

The secondary marketplace for names. Names are automatically listed here once they are minted.

### [Whitelist](./contracts/whitelist-updatable/README.md)

Whitelist allows for flexible updating to add / remove addresses at any point in minting. Also adds helper to account for whitelist minting limits.

## DISCLAIMER

STARGAZE SOURCE CODE IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating or instantiating Stargaze smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of Stargaze, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value. Although Public Awesome, LLC and it's affilliates developed the initial code for Stargaze, it does not own or control the Stargaze network, which is run by a decentralized validator set.
