use soroban_sdk::{Address, Env, String};
use crate::types::Collectible;
use crate::storage::{increment_count, write_collectible};

pub fn mint(env: &Env, admin: &Address, name: String, site: String, price: i128, artist: Address) -> u64 {
    admin.require_auth();

    let id = increment_count(env);
    let collectible = Collectible {
        id,
        name,
        site,
        price,
        artist,
        owner: None,
    };

    write_collectible(env, id, &collectible);
    id
}
