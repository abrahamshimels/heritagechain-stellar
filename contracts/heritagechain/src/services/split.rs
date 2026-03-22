pub fn split_payment(amount: i128) -> (i128, i128, i128) {
    let treasury_share = (amount * 70) / 100;
    let site_share = (amount * 20) / 100;
    let artist_share = amount - treasury_share - site_share; // 10%
    
    (treasury_share, site_share, artist_share)
}
