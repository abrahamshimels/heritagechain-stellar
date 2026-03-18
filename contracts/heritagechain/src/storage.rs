use soroban_sdk::{Address, Env, Vec};
use crate::types::{Collectible, DataKey};

pub fn read_collectible(env: &Env, id: u64) -> Option<Collectible> {
    env.storage().persistent().get(&DataKey::Collectible(id))
}

pub fn write_collectible(env: &Env, id: u64, collectible: &Collectible) {
    env.storage().persistent().set(&DataKey::Collectible(id), collectible);
}

pub fn read_user_collection(env: &Env, user: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::UserCollection(user.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn write_user_collection(env: &Env, user: &Address, collection: &Vec<u64>) {
    env.storage().persistent().set(&DataKey::UserCollection(user.clone()), collection);
}

pub fn read_count(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::CollectibleCount).unwrap_or(0)
}

pub fn increment_count(env: &Env) -> u64 {
    let count = read_count(env) + 1;
    env.storage().instance().set(&DataKey::CollectibleCount, &count);
    count
}
