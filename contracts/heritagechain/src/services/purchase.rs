use soroban_sdk::{Address, Env};
use crate::storage::{read_collectible, read_user_collection, write_collectible, write_user_collection};
use crate::services::split::execute_split;

pub fn purchase(
    env: &Env,
    buyer: &Address,
    collectible_id: u64,
    token: &Address,
    treasury: &Address,
    site_fund: &Address,
) {
    buyer.require_auth();

    let mut collectible = read_collectible(env, collectible_id).expect("collectible not found");
    if collectible.owner.is_some() {
        panic!("already owned");
    }

    // execute payments
    execute_split(
        env,
        token,
        buyer,
        collectible.price,
        treasury,
        site_fund,
        &collectible.artist,
    );

    // Update ownership
    collectible.owner = Some(buyer.clone());
    write_collectible(env, collectible_id, &collectible);

    // Update user collection
    let mut collection = read_user_collection(env, buyer);
    collection.push_back(collectible_id);
    write_user_collection(env, buyer, &collection);
}
