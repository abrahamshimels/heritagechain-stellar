use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use crate::types::Collectible;
use crate::storage::{get_collectible, get_user_collection, get_count};
use crate::services::mint::mint_collectible as mint;
use crate::services::purchase::purchase_collectible as purchase;

#[contract]
pub struct HeritageChain;

#[contractimpl]
impl HeritageChain {
    pub fn mint_collectible(
        env: Env,
        admin: Address,
        name: String,
        site: String,
        price: i128,
        artist: Address,
    ) -> u64 {
        mint(&env, admin, name, site, price, artist)
    }

    pub fn purchase_collectible(
        env: Env,
        buyer: Address,
        collectible_id: u64,
        token: Address,
        treasury: Address,
        site_fund: Address,
    ) {
        purchase(&env, buyer, collectible_id, token, treasury, site_fund)
    }

    pub fn get_collectibles(env: Env) -> Vec<Collectible> {
        let count = get_count(&env);
        let mut list = Vec::new(&env);
        for id in 1..=count {
            if let Some(col) = get_collectible(&env, id) {
                list.push_back(col);
            }
        }
        list
    }

    pub fn get_user_collection(env: Env, user: Address) -> Vec<Collectible> {
        let ids = get_user_collection(&env, user);
        let mut list = Vec::new(&env);
        for id in ids.iter() {
            if let Some(col) = get_collectible(&env, id) {
                list.push_back(col);
            }
        }
        list
    }
}
