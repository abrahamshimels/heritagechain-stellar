use soroban_sdk::{Address, Env};
use soroban_sdk::token::Client as TokenClient;
use crate::storage::{get_collectible, save_collectible, add_to_user, has_collectible};
use crate::services::split::split_payment;

pub fn purchase_collectible(
    env: &Env,
    buyer: Address,
    collectible_id: u64,
    token: Address,
    treasury: Address,
    site_fund: Address,
) {
    buyer.require_auth();

    // 1. Get collectible
    let mut collectible = get_collectible(env, collectible_id).expect("Collectible not found");
    
    // 2. Check if user does NOT already own it (per requirements)
    if has_collectible(env, buyer.clone(), collectible_id) {
        panic!("User already owns this collectible");
    }

    // 3. Call split_payment (Pure function)
   let split = split_payment(collectible.price);
let treasury_share = split.treasury_amount;
let site_share = split.site_fund_amount;
let artist_share = split.artist_amount;

    // 4. Perform token transfers
    let client = TokenClient::new(env, &token);
    client.transfer(&buyer, &treasury, &treasury_share);
    client.transfer(&buyer, &site_fund, &site_share);
    client.transfer(&buyer, &collectible.artist, &artist_share);

    // 5. Update ownership
    collectible.owner = buyer.clone();
    save_collectible(env, collectible_id, &collectible);
    
    // 6. Record in user collection
    add_to_user(env, buyer, collectible_id);
}
