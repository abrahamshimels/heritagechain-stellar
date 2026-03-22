use soroban_sdk::{Address, Env, Vec};
use crate::types::{Collectible, DataKey};

pub fn save_collectible(env: &Env, id: u64, collectible: &Collectible) {
    env.storage().persistent().set(&DataKey::Collectible(id), collectible);
}

pub fn get_collectible(env: &Env, id: u64) -> Option<Collectible> {
    env.storage().persistent().get(&DataKey::Collectible(id))
}

pub fn add_to_user(env: &Env, user: Address, id: u64) {
    let mut collection: Vec<u64> = env.storage()
        .persistent()
        .get(&DataKey::UserCollection(user.clone()))
        .unwrap_or_else(|| Vec::new(env));
    
    collection.push_back(id);
    env.storage().persistent().set(&DataKey::UserCollection(user), &collection);
}

pub fn has_collectible(env: &Env, user: Address, id: u64) -> bool {
    let collection: Vec<u64> = env.storage()
        .persistent()
        .get(&DataKey::UserCollection(user))
        .unwrap_or_else(|| Vec::new(env));
    
    collection.contains(id)
}

pub fn get_next_id(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::CollectibleCount).unwrap_or(0) + 1
}

pub fn increment_id(env: &Env) {
    let next = get_next_id(env);
    env.storage().instance().set(&DataKey::CollectibleCount, &next);
}

pub fn get_count(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::CollectibleCount).unwrap_or(0)
}

pub fn get_user_collection(env: &Env, user: Address) -> Vec<u64> {
     env.storage()
        .persistent()
        .get(&DataKey::UserCollection(user))
        .unwrap_or_else(|| Vec::new(env))
}
