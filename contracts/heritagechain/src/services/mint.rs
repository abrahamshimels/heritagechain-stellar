use soroban_sdk::{Address, Env, String};
use crate::types::Collectible;
use crate::storage::{get_next_id, increment_id, save_collectible};

pub fn mint_collectible(
    env: &Env,
    admin: Address,
    name: String,
    site: String,
    price: i128,
    artist: Address,
) -> u64 {
    admin.require_auth();

    let id = get_next_id(env);
    let collectible = Collectible {
        id,
        name,
        site,
        price,
        artist,
        owner: admin.clone(), // Admin is the initial owner per requirements
    };

    save_collectible(env, id, &collectible);
    increment_id(env);
    
    id
}
