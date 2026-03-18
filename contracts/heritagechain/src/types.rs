use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Collectible {
    pub id: u64,
    pub name: String,
    pub site: String,
    pub price: i128,
    pub artist: Address,
    pub owner: Option<Address>,
}

#[contracttype]
pub enum DataKey {
    Collectible(u64),
    UserCollection(Address),
    CollectibleCount,
}
