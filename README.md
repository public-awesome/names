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

## Contracts

### sg721-name

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
    pub content: String,
    pub profile: NFT,
    pub record: Vec<TextRecord>,
    pub extension: T,
}
```

### Name Minter

The top-level name minter is initialized with the prefix `stars`.

```rs
pub struct InitializeMsg {
    pub name: String,
}
```

The mint function mints a new name as the `token_id`.

```rs
pub enum ExecuteMsg {
    Mint { name: String }
}
```

A name minter will also keep a mapping of addresses to names for reverse lookups:

```rs
/// Addr = address associated with name
/// String = the name (token_id)
pub const ADDRESS_NAME: Map<&Addr, String> = Map::new("a");
```
