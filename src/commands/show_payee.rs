//! Show merchant account details

use crate::config::TallyCliConfig;
use crate::utils::colors::Theme;
use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::SimpleTallyClient;

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
    let merchant_address =
        Pubkey::from_str(request.merchant).context("Failed to parse merchant public key")?;

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
        // Human-readable output with colors
        let mut output = String::new();
        writeln!(&mut output, "{}", Theme::header("Merchant Account Details"))?;
        writeln!(&mut output, "{}", Theme::dim("========================"))?;
        writeln!(&mut output, "{:<18} {}", Theme::info("Merchant PDA:"), Theme::highlight(request.merchant))?;
        writeln!(&mut output, "{:<18} {}", Theme::info("Authority:"), Theme::value(&merchant.authority.to_string()))?;
        writeln!(&mut output, "{:<18} {}", Theme::info("USDC Mint:"), Theme::dim(&merchant.usdc_mint.to_string()))?;
        writeln!(&mut output, "{:<18} {}", Theme::info("Treasury ATA:"), Theme::value(&merchant.treasury_ata.to_string()))?;
        writeln!(&mut output, "{:<18} {} bps ({}%)",
            Theme::info("Platform Fee:"),
            merchant.platform_fee_bps,
            Theme::value(&config.format_fee_percentage(merchant.platform_fee_bps).to_string()))?;
        writeln!(&mut output, "{:<18} {} ({})",
            Theme::info("Tier:"),
            merchant.tier,
            Theme::active(tier_name(merchant.tier)))?;
        write!(&mut output, "{:<18} {}", Theme::info("Bump:"), Theme::dim(&merchant.bump.to_string()))?;
        Ok(output)
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
