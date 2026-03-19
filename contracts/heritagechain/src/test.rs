#![cfg(test)]
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use crate::contract::{HeritageChain, HeritageChainClient};

#[test]
fn test_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let artist = Address::generate(&env);
    
    let contract_id = env.register_contract(None, HeritageChain);
    let client = HeritageChainClient::new(&env, &contract_id);

    // 1. Mint
    let name = String::from_str(&env, "Test Item");
    let site = String::from_str(&env, "Test Site");
    let price = 1000i128;
    
    let id = client.mint_collectible(&admin, &name, &site, &price, &artist);
    assert_eq!(id, 1);

    // 2. Fetch
    let collectibles = client.get_collectibles();
    assert_eq!(collectibles.len(), 1);
    let item = collectibles.get(0).unwrap();
    assert_eq!(item.name, name);
    assert_eq!(item.owner, admin);
}
