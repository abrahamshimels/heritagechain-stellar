use soroban_sdk::{Address, Env};
use soroban_sdk::token::Client as TokenClient;

pub fn execute_split(
    env: &Env,
    token: &Address,
    buyer: &Address,
    price: i128,
    treasury: &Address,
    site_fund: &Address,
    artist: &Address,
) {
    let treasury_amt = (price * 70) / 100;
    let site_fund_amt = (price * 20) / 100;
    let artist_amt = price - treasury_amt - site_fund_amt; // remaining 10%

    let client = TokenClient::new(env, token);
    client.transfer(buyer, treasury, &treasury_amt);
    client.transfer(buyer, site_fund, &site_fund_amt);
    client.transfer(buyer, artist, &artist_amt);
}
