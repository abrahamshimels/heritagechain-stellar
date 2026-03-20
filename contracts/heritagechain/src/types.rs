use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Collectible {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub site: String,
    pub edition: String,
    pub price: i128,
    pub artist: Address,
    pub owner: Address,
    pub is_sold: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Collectible(u64),
    UserCollection(Address),
    CollectibleCount,
}
