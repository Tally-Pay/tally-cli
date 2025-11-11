//! Show payee account details

use crate::config::TallyCliConfig;
use crate::utils::colors::Theme;
use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::{BasisPoints, SimpleTallyClient, UsdcAmount};

/// Request to show payee details
pub struct ShowPayeeRequest<'a> {
    /// Payee PDA address
    pub payee: &'a str,
    /// Output format
    pub output_format: &'a str,
}

/// Execute the show-payee command
///
/// # Arguments
/// * `tally_client` - The Tally SDK client
/// * `request` - The show payee request parameters
/// * `config` - CLI configuration
///
/// # Returns
/// * `Ok(String)` - Formatted payee details
///
/// # Errors
/// Returns an error if:
/// * Payee public key cannot be parsed
/// * Failed to fetch payee account from RPC
/// * Payee account not found
/// * JSON serialization fails
///
/// # Panics
/// May panic if platform fee basis points exceed maximum allowed value (10000).
/// This should not occur under normal operation as values are validated on-chain.
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &ShowPayeeRequest<'_>,
    _config: &TallyCliConfig,
) -> Result<String> {
    // Parse payee address
    let payee_address =
        Pubkey::from_str(request.payee).context("Failed to parse payee public key")?;

    // Fetch payee account
    let payee = tally_client
        .get_payee(&payee_address)
        .context("Failed to fetch payee account - check RPC connection and account state")?
        .context("Payee account not found")?;

    // Create type-safe values
    let monthly_volume = UsdcAmount::from_microlamports(payee.monthly_volume_usdc);
    let platform_fee = BasisPoints::new(volume_tier_fee_bps(payee.volume_tier))
        .expect("Valid platform fee");

    // Format output based on requested format
    if request.output_format == "json" {
        let json_output = serde_json::json!({
            "payee": request.payee,
            "authority": payee.authority.to_string(),
            "usdc_mint": payee.usdc_mint.to_string(),
            "treasury_ata": payee.treasury_ata.to_string(),
            "volume_tier": volume_tier_name(payee.volume_tier),
            "monthly_volume_microlamports": monthly_volume.microlamports(),
            "monthly_volume_usdc": monthly_volume.usdc(),
            "monthly_volume_display": monthly_volume.to_string(),
            "platform_fee_bps": platform_fee.raw(),
            "platform_fee_pct": platform_fee.percentage(),
            "platform_fee_display": platform_fee.to_string(),
            "last_volume_update_ts": payee.last_volume_update_ts,
            "bump": payee.bump,
        });
        Ok(serde_json::to_string_pretty(&json_output)?)
    } else {
        // Human-readable output with colors
        let mut output = String::new();
        writeln!(&mut output, "{}", Theme::header("Payee Account Details"))?;
        writeln!(&mut output, "{}", Theme::dim("======================"))?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Payee PDA:"),
            Theme::highlight(request.payee)
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Authority:"),
            Theme::value(&payee.authority.to_string())
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("USDC Mint:"),
            Theme::dim(&payee.usdc_mint.to_string())
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Treasury ATA:"),
            Theme::value(&payee.treasury_ata.to_string())
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Volume Tier:"),
            Theme::active(volume_tier_name(payee.volume_tier))
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Monthly Volume:"),
            monthly_volume
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Platform Fee:"),
            Theme::value(&platform_fee.to_string())
        )?;
        write!(
            &mut output,
            "{:<22} {}",
            Theme::info("Bump:"),
            Theme::dim(&payee.bump.to_string())
        )?;
        Ok(output)
    }
}

/// Convert volume tier to human-readable name
const fn volume_tier_name(tier: tally_sdk::program_types::VolumeTier) -> &'static str {
    use tally_sdk::program_types::VolumeTier;
    match tier {
        VolumeTier::Standard => "Standard",
        VolumeTier::Growth => "Growth",
        VolumeTier::Scale => "Scale",
    }
}

/// Get platform fee in basis points for volume tier
const fn volume_tier_fee_bps(tier: tally_sdk::program_types::VolumeTier) -> u16 {
    use tally_sdk::program_types::VolumeTier;
    match tier {
        VolumeTier::Standard => 25, // 0.25%
        VolumeTier::Growth => 20,   // 0.20%
        VolumeTier::Scale => 15,    // 0.15%
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_tier_name() {
        use tally_sdk::program_types::VolumeTier;
        assert_eq!(volume_tier_name(VolumeTier::Standard), "Standard");
        assert_eq!(volume_tier_name(VolumeTier::Growth), "Growth");
        assert_eq!(volume_tier_name(VolumeTier::Scale), "Scale");
    }

    #[test]
    fn test_volume_tier_fee_bps() {
        use tally_sdk::program_types::VolumeTier;
        assert_eq!(volume_tier_fee_bps(VolumeTier::Standard), 25);
        assert_eq!(volume_tier_fee_bps(VolumeTier::Growth), 20);
        assert_eq!(volume_tier_fee_bps(VolumeTier::Scale), 15);
    }
}
