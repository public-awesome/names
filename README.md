# Stargaze Names

## Initial Fees

```
5+ chars = 100 STARS
4 chars = 1,000 STARS
3 chars = 10,000 STARS
```

## Annual Auction

- When a name is minted it is automatically listed in Name Marketplace
- Owner can accept the top bid at any time
- After 1 year, owner has to pay 0.5% of the top bid to keep the name
- If a bid is placed for 4 weeks, name value rises to this value
- If fee is not paid, name is transferred to the bidder
- If there are no bids, there is a minimum fee to keep the name based on the number of characters
- Cap annual fee at X per year

## Overview

A name points to one address. An address points to one name. A name may have any amount of subnames with different owners. A single owner may own multiple names.

You can only associate a name with an address for which you own. DAOs and multisigs can own names.

```
name: jeanluc.stars
address: stars157nwwthsyyskuzeugsf48k52v6s4sl2swlhm2r
owner: stars157nwwthsyyskuzeugsf48k52v6s4sl2swlhm2r
```

```
name: spaceships.stars
address: stars1lhz29slmz60lskr9yf8c3wn3p344n9g4jz88wx1h2gf322g3hf
owner: stars157nwwthsyyskuzeugsf48k52v6s4sl2swlhm2r
```

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

## Contracts

### SG-721 Name (sg721-name)

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

### Name Minter (name-minter)

Name minter is responsible for minting, validating, and updating names and their metadata.

### Name Marketplace (marketplace)

The secondary marketplace for names. Names are automatically listed here once they are minted.
