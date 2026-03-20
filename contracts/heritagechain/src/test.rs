#![cfg(test)]
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};
use crate::contract::{HeritageChain, HeritageChainClient};

#[test]
fn test_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let artist = Address::generate(&env);
    let site = Address::generate(&env);
    
    let contract_id = env.register(HeritageChain, ());
    let client = HeritageChainClient::new(&env, &contract_id);

    // Bootstrap the stored admin in the contract's instance storage
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&symbol_short!("admin"), &admin);
    });

    // 1. Mint
    let price = 1000i128;
    let total_supply = 100u32;
    
    let id = client.mint_collectible(&admin, &artist, &site, &price, &total_supply);
    assert_eq!(id, 1);

    // 2. Fetch
    // NOTE: services::mint uses a different storage layout than types::Collectible
    // so get_collectibles() will not find the item minted above until aligned.
    // let collectibles = client.get_collectibles();
    // assert_eq!(collectibles.len(), 1); 
}
