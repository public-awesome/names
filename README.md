# Stargaze Names

## DUAL LICENSED

The code is this repository is under two licenses. Please make a note of them before forking / copying.

[Name Marketplace](./contracts/marketplace/LICENSE) -- Business source license

[Name Minter](./contracts/name-minter/LICENSE) -- Apache 2.0 license

[SG721 Name](./contracts/sg721-name/LICENSE) -- Apache 2.0 license

## Annual Auction

- When a name is minted it is automatically listed in Name Marketplace (x)
- Owner can accept the top bid at any time (x)
- After 1 year, owner has to pay 0.5% of the top bid to keep the name
- If a bid is placed for 4 weeks, name value rises to this value
- If fee is not paid, name is transferred to the bidder
- If there are no bids, there is a minimum fee to keep the name based on the number of characters
- Cap annual fee at X per year

## Architecture

Names are stored without the TLD so they can be mapped to a raw address that is not bech32 encoded. This way, all Cosmos / Interchain names can be resolved to an address that is derived via the same key derivation path (118).

For example:

```
bobo -> D93385094E906D7DA4EBFDEC2C4B167D5CAA431A (in hex)
```

Now this can be resolved per chain:

```
bobo.stars -> stars1myec2z2wjpkhmf8tlhkzcjck04w25sc6ymhplz
bobo.cosmos -> cosmos1myec2z2wjpkhmf8tlhkzcjck04w25sc6y2xq2r
```

This architecture enables Stargaze Names to be a truly Interchain name service since it can mint and resolve names for any Cosmos chain.

## Initial Fees

```
5+ chars = 500 STARS
4 chars = 5,000 STARS
3 chars = 50,000 STARS
```

## Contracts

### [SG-721 Name](./contracts/sg721-name/README.md)

A cw721 contract with on-chain metadata for a name.

Types used in metadata:

```rs
pub struct NFT {
    pub collection: Addr,
    pub token_id: String,
}

pub struct TextRecord {
    pub name: String,  // "twitter"
    pub value: String, // "shan3v"
    pub verified_at: Option<Timestamp>
}
```

```rs
pub struct Metadata {
    pub bio: String,
    pub profile: NFT,
    pub record: Vec<TextRecord>,
    pub extension: T,
}
```

### [Name Minter](./contracts/name-minter/README.md)

Name minter is responsible for minting, validating, and updating names and their metadata.

### [Name Marketplace](./contracts/marketplace/README.md)

The secondary marketplace for names. Names are automatically listed here once they are minted.

## DISCLAIMER

STARGAZE NAME MARKETPLACE IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating or instantiating Stargaze smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of Stargaze, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value. Although Public Awesome, LLC and it's affilliates developed the initial code for Stargaze, it does not own or control the Stargaze network, which is run by a decentralized validator set.
