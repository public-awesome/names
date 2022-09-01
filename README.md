# Stargaze Names

## Initial Fees

```
5+ chars = 1,000 STARS
4 chars = 10,000 STARS
3 chars = 100,000 STARS
2 chars = 1,000,000 STARS // save for later
1 char = 10,000,000 STARS // save for later
```

## Annual Auction

- When a name is minted it is automatically listed in Name Marketplace
- Owner can accept the top bid at any time
- After 1 year, owner has to pay 3% of the top bid to keep the name
- There is a 30 day grace period after name expiry for owner to pay fee
- If owner doesn't pay the fee during the grace period, anyone can claim the name for the highest bid amount
- If there are no bids, there is a minimum fee to keep the name based on the number of characters
- Cap annual fee at $100 / year
- Cap fee growth rate for N years
- If the name is a subname, the root name owner gets 50% of the fee

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

## Subnames

A name may have any amount of subnames with different owners.

Subname issuers are basically registrars, and it is recommended (especially very short name) issuers do it through a DAO.

```
name: stargazer.spaceships.stars
address: stars1ftdlahh47zuwz73832xf89gfv53xxy2eh3u56f
owner: stars1ftdlahh47zuwz73832xf89gfv53xxy2eh3u56f
```

```
name: defiant.spaceships.stars
address: stars14vkuqqqtr99r3y4pacqx43e6d8rfa8nea8mtv4
owner: stars1ftdlahh47zuwz73832xf89gfv53xxy2eh3u56f
```

Subnames can also have subnames, ad infinitum.

```
name: uss.stargazer.spaceships.stars
address: stars1ue3vm5fvxsa9qkvs37r6x3jsx7suaqg4sx8yhm
owner: stars1ftdlahh47zuwz73832xf89gfv53xxy2eh3u56f
```

## Architecture

Stargaze Names consists of a top-level name minter that mints names to itself, since its a cw721. It can also instantiate another instance of itself to mint subnames. Because both contracts are minters, they implement the same `NameMinter` trait.

```rs
pub trait NameMinter {
    fn init(parent_name: String);
    fn mint(name: String) -> String;
}
```

- The top-level minter is initialized with a parent name of `.stars`.
- `Mint {}` mints a name like `badkids.stars` and stores it in this top-level contract.
- `InitSubnames {}` initializes a new name minter with the parent name of `badkids.stars`.
- `Mint {}` on this `badkids.stars` contract mints subnames like `shane.badkids.stars`.
- `InitSubnames {}` initialized a new name minter with the parent name of `shane.badkids.stars`.
- `Mint {}` on this `shane.badkids.stars` contract mints subnames like `newyork.shane.badkids.stars`.

## Contracts

### sg721-sub

A cw721-compatible contract for NFT subscriptions.

Extension to `TokenInfo`:

```rs
pub struct RenewalExtension<T> {
    pub expiration: Timestamp,
    pub extenstion: T,
}
```

Message additions:

```rs
pub enum ExecuteMsg {
    /// Renew the subscription by paying all or a partial amount.
    /// Only the final payer becomes the new owner.
    Renew { token_id: String },
    /// Someone may call this to take ownership and of an expired subscription
    Claim { token_id: String },
}
```

When `Renew` is called, another year is added to the `expiration` time.

### Name Minter

A name minter is initialized with the parent name, such as `stars`, or `badkids.stars`.

```rs
pub struct InitializeMsg {
    pub parent_name: String,
}
```

The mint function mints a new name (`token_id`) as a subname. For example, `badkids.stars`, or `shane.badkids.stars`.

```rs
pub enum ExecuteMsg {
    Mint { name: String }
}
```

Name minter mints subscription NFTs with a name metadata extension:

```rs
pub struct NameMetadataExtension<T> {
    /// Address associated with the name.
    pub address: Addr,
    pub content: String,
    pub profile: NFT,
    pub record: Vec<TextRecord>,
    pub extension: T,
}

pub struct NFT {
    pub collection: Addr,
    pub token_id: String,
}

pub struct TextRecord {
    pub name: String,  // "twitter"
    pub value: String, // "shan3v"
}
```

Updating `content` or adding new records should charge fees to prevent storage bloat, and they should be validated for length.

So the extension is:

```rs
let extension = RenewalExtension<NameMetadataExtesion>
```

A name minter will also keep a mapping of addresses to names for reverse lookups:

```rs
/// Addr = address associated with name
/// String = the name (token_id)
pub const ADDRESSES: Map<&Addr, String> = Map::new("a");
```
