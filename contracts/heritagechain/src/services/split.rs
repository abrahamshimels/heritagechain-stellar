//! Payment Split Engine for HeritageChain on Stellar
//! 
//! This module handles automatic revenue distribution for collectible purchases.
//! Split ratios: 70% Government Treasury / 20% Site Preservation / 10% Artist

// ============================================================================
// CONSTANTS
// ============================================================================

/// Standard split percentages for digital collectibles
pub const TREASURY_PCT: i128 = 70;
pub const SITE_FUND_PCT: i128 = 20;
pub const ARTIST_PCT: i128 = 10;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Result of splitting a digital collectible payment
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitResult {
    pub treasury_amount: i128,
    pub site_fund_amount: i128,
    pub artist_amount: i128,
}

// ============================================================================
// CORE SPLIT FUNCTIONS
// ============================================================================

/// Split a digital collectible payment (70/20/10)
/// 
/// # Arguments
/// * `amount` - Total payment amount in stroops (10^7 per token)
/// 
/// # Returns
/// * `SplitResult` - (treasury, site_fund, artist) amounts
/// 
/// # Example
/// ```
/// let result = split_payment(10_000_000); // 1 token
/// assert_eq!(result.treasury_amount, 7_000_000);
/// assert_eq!(result.site_fund_amount, 2_000_000);
/// assert_eq!(result.artist_amount, 1_000_000);
/// ```
pub fn split_payment(amount: i128) -> SplitResult {
    let treasury_amount = (amount * TREASURY_PCT) / 100;
    let site_fund_amount = (amount * SITE_FUND_PCT) / 100;
    // Calculate artist as remainder to avoid rounding errors
    let artist_amount = amount - treasury_amount - site_fund_amount;
    
    SplitResult {
        treasury_amount,
        site_fund_amount,
        artist_amount,
    }
}

/// Validate that a split result is mathematically correct
pub fn validate_split(result: &SplitResult, original_amount: i128) -> bool {
    result.treasury_amount + result.site_fund_amount + result.artist_amount == original_amount
}

/// Get the treasury percentage
pub fn get_treasury_percentage() -> i128 {
    TREASURY_PCT
}

/// Get the site fund percentage
pub fn get_site_fund_percentage() -> i128 {
    SITE_FUND_PCT
}

/// Get the artist percentage
pub fn get_artist_percentage() -> i128 {
    ARTIST_PCT
}

/// Get human-readable description of the split ratios
pub fn get_split_description() -> &'static str {
    "Digital Collectible Split:\n\
     ┌─────────────────────────────────────┐\n\
     │ Government Treasury:  70%           │\n\
     │ Site Preservation:    20%           │\n\
     │ Artist Royalties:     10%           │\n\
     └─────────────────────────────────────┘"
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    // 1 token = 10,000,000 stroops (7 decimals on Stellar)
    const ONE_TOKEN: i128 = 10_000_000;
    
    #[test]
    fn test_split_payment_basic() {
        let result = split_payment(ONE_TOKEN);
        
        assert_eq!(result.treasury_amount, 7_000_000);
        assert_eq!(result.site_fund_amount, 2_000_000);
        assert_eq!(result.artist_amount, 1_000_000);
        assert!(validate_split(&result, ONE_TOKEN));
    }
    
    #[test]
    fn test_split_payment_small_amount() {
        let payment = 1000;
        let result = split_payment(payment);
        
        assert_eq!(result.treasury_amount, 700);
        assert_eq!(result.site_fund_amount, 200);
        assert_eq!(result.artist_amount, 100);
        assert_eq!(result.treasury_amount + result.site_fund_amount + result.artist_amount, payment);
    }
    
    #[test]
    fn test_split_payment_rounding() {
        // Test with amount that doesn't divide evenly by 10
        let payment = 999;
        let result = split_payment(payment);
        
        assert_eq!(result.treasury_amount, 699);
        assert_eq!(result.site_fund_amount, 199);
        assert_eq!(result.artist_amount, 101); // 999 - 699 - 199 = 101
        assert_eq!(result.treasury_amount + result.site_fund_amount + result.artist_amount, payment);
    }
    
    #[test]
    fn test_zero_payment() {
        let result = split_payment(0);
        assert_eq!(result.treasury_amount, 0);
        assert_eq!(result.site_fund_amount, 0);
        assert_eq!(result.artist_amount, 0);
        assert!(validate_split(&result, 0));
    }
    
    #[test]
    fn test_large_payment() {
        let payment = 1_000_000_000_000_000i128; // Very large amount
        let result = split_payment(payment);
        
        assert_eq!(result.treasury_amount, 700_000_000_000_000);
        assert_eq!(result.site_fund_amount, 200_000_000_000_000);
        assert_eq!(result.artist_amount, 100_000_000_000_000);
        assert!(validate_split(&result, payment));
    }
    
    #[test]
    fn test_validate_split_correct() {
        let result = SplitResult {
            treasury_amount: 700,
            site_fund_amount: 200,
            artist_amount: 100,
        };
        assert!(validate_split(&result, 1000));
    }
    
    #[test]
    fn test_validate_split_incorrect() {
        let result = SplitResult {
            treasury_amount: 700,
            site_fund_amount: 200,
            artist_amount: 101, // 1 too much
        };
        assert!(!validate_split(&result, 1000));
    }
    
    #[test]
    fn test_get_split_description() {
        let desc = get_split_description();
        assert!(desc.contains("70%"));
        assert!(desc.contains("20%"));
        assert!(desc.contains("10%"));
    }
    
    #[test]
    fn test_get_percentages() {
        assert_eq!(get_treasury_percentage(), 70);
        assert_eq!(get_site_fund_percentage(), 20);
        assert_eq!(get_artist_percentage(), 10);
    }
    
    #[test]
    fn test_split_payment_preserves_total() {
        // Use an array instead of vec to avoid vec! macro
        let test_amounts = [1, 10, 100, 999, 1000, 10_000, 100_000, ONE_TOKEN];
        
        for amount in test_amounts.iter() {
            let result = split_payment(*amount);
            let total = result.treasury_amount + result.site_fund_amount + result.artist_amount;
            assert_eq!(total, *amount, "Split total should equal original amount for {}", amount);
        }
    }
}