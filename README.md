# Stargaze Names

## Overview

A name points to one address. An address points to one name. A name may have any amount of subnames with different owners. A single owner may own multiple names.

You can only associate a name with your own address, or a contract address you are the admin of.

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

Each collection is an implementation of cw721 that supports subscriptions. Minting a name is like buying a subscription to that name for one year. The fee can be paid by anybody. When a new name is minted, its expiration time, renewal amount, and renewal amount collected so far, is tracked in `TokenInfo`. Anyone may call `Claim {}` after expiration to claim a name for themselves, as long as they pay the renewal fee. After expiration, anyone may also call `Burn {}` and collect any fees paid against the renewal.

## Contracts

### SG721-SUB

A cw721-compatible contract for NFT subscriptions.

Extension to `TokenInfo`:

```rs
pub struct RenewalExtension<T> {
    pub expiration: Timestamp,
    pub cost: Coin,
    pub collected: Coin,
    pub extension: T,
}
```

`RenewalExtension` itself can take an extension for further extensibility.

Message additions:

```rs
pub enum ExecuteMsg {
    /// Renew the subscription by paying all or a partial amount.
    /// Only the final payer becomes the new owner.
    Renew { token_id: String },
    /// Someone may call this to take ownership and of an expired subscription
    Claim { token_id: String },
    /// Someone may call this to burn and collect fees from an expired subscription
    Burn { token_id: String },
}
```

When `Renew` is called, another year is added to the `expiration` time, and `cost` and `collected` are updated. The collected fee is processed, going to name owner with a Fair Burn amount.

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
    /// Address associated with the name. Doesn't have to be the owner. For example, this could be a collection contract address.
    pub address: Addr,
    pub content: String,
    pub record: Vec<TextRecord>,
    pub extension: T,
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

## Fees

```
5+ chars = 1,000 STARS
4 chars = 10,000 STARS
3 chars = 100,000 STARS
```

Names have to be renewed every year. Parent name owner is allowed to charge annual fees. 95% goes to name owner, 5% is Fair Burned. You can pre-pay for up to 5 years at once. For the top-level name `.stars`, fees go to the Community Pool.
