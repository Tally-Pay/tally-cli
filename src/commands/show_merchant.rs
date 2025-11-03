//! Show merchant account details

use anyhow::{Context, Result};
use tally_sdk::SimpleTallyClient;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::config::TallyCliConfig;

/// Request to show merchant details
pub struct ShowMerchantRequest<'a> {
    /// Merchant PDA address
    pub merchant: &'a str,
    /// Output format
    pub output_format: &'a str,
}

/// Execute the show-merchant command
///
/// # Arguments
/// * `tally_client` - The Tally SDK client
/// * `request` - The show merchant request parameters
/// * `config` - CLI configuration
///
/// # Returns
/// * `Ok(String)` - Formatted merchant details
///
/// # Errors
/// Returns an error if:
/// * Merchant public key cannot be parsed
/// * Failed to fetch merchant account from RPC
/// * Merchant account not found
/// * JSON serialization fails
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &ShowMerchantRequest<'_>,
    config: &TallyCliConfig,
) -> Result<String> {
    // Parse merchant address
    let merchant_address = Pubkey::from_str(request.merchant)
        .context("Failed to parse merchant public key")?;

    // Fetch merchant account
    let merchant = tally_client
        .get_merchant(&merchant_address)
        .context("Failed to fetch merchant account - check RPC connection and account state")?
        .context("Merchant account not found")?;

    // Format output based on requested format
    if request.output_format == "json" {
        let json_output = serde_json::json!({
            "merchant": request.merchant,
            "authority": merchant.authority.to_string(),
            "usdc_mint": merchant.usdc_mint.to_string(),
            "treasury_ata": merchant.treasury_ata.to_string(),
            "platform_fee_bps": merchant.platform_fee_bps,
            "platform_fee_pct": config.format_fee_percentage(merchant.platform_fee_bps),
            "tier": merchant.tier,
            "tier_name": tier_name(merchant.tier),
            "bump": merchant.bump,
        });
        Ok(serde_json::to_string_pretty(&json_output)?)
    } else {
        // Human-readable output
        Ok(format!(
            "Merchant Account Details
========================
Merchant PDA:      {}
Authority:         {}
USDC Mint:         {}
Treasury ATA:      {}
Platform Fee:      {} ({})
Tier:              {} ({})
Bump:              {}
",
            request.merchant,
            merchant.authority,
            merchant.usdc_mint,
            merchant.treasury_ata,
            merchant.platform_fee_bps,
            config.format_fee_percentage(merchant.platform_fee_bps),
            merchant.tier,
            tier_name(merchant.tier),
            merchant.bump,
        ))
    }
}

/// Convert tier number to human-readable name
const fn tier_name(tier: u8) -> &'static str {
    match tier {
        0 => "Free",
        1 => "Pro",
        2 => "Enterprise",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_name() {
        assert_eq!(tier_name(0), "Free");
        assert_eq!(tier_name(1), "Pro");
        assert_eq!(tier_name(2), "Enterprise");
        assert_eq!(tier_name(99), "Unknown");
    }
}
