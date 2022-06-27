use cosmwasm_std::Addr;

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
