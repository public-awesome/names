# Stargaze Names Collection

Defines on-chain metadata for Stargaze Names, a Cosmos / Interchain name service.

```rs
pub struct NFT {
    pub collection: Addr,
    pub token_id: String,
}

pub struct TextRecord {
    pub name: String,  // "twitter"
    pub value: String, // "shan3v"
    pub verified_at: Option<Timestamp>,
}

pub struct Metadata {
    pub image: Option<NFT>,
    pub records: Vec<TextRecord>,
}
```
