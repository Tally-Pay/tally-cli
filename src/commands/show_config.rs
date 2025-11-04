//! Show global configuration account details

use crate::config::TallyCliConfig;
use crate::utils::colors::Theme;
use anyhow::{Context, Result};
use std::fmt::Write as _;
use tally_sdk::SimpleTallyClient;

/// Request to show config details
pub struct ShowConfigRequest<'a> {
    /// Output format
    pub output_format: &'a str,
}

/// Execute the show-config command
///
/// # Arguments
/// * `tally_client` - The Tally SDK client
/// * `request` - The show config request parameters
/// * `config` - CLI configuration
///
/// # Returns
/// * `Ok(String)` - Formatted config details
///
/// # Errors
/// Returns an error if:
/// * Failed to fetch config account from RPC
/// * Config account not found
/// * JSON serialization fails
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &ShowConfigRequest<'_>,
    config: &TallyCliConfig,
) -> Result<String> {
    // Fetch config account
    let cfg = tally_client
        .get_config()
        .context("Failed to fetch config account - check RPC connection and account state")?
        .context("Config account not found - has init-config been run?")?;

    // Format output based on requested format
    if request.output_format == "json" {
        let json_output = serde_json::json!({
            "platform_authority": cfg.platform_authority.to_string(),
            "pending_authority": cfg.pending_authority.map(|p| p.to_string()),
            "max_platform_fee_bps": cfg.max_platform_fee_bps,
            "max_platform_fee_pct": config.format_fee_percentage(cfg.max_platform_fee_bps),
            "min_platform_fee_bps": cfg.min_platform_fee_bps,
            "min_platform_fee_pct": config.format_fee_percentage(cfg.min_platform_fee_bps),
            "min_period_seconds": cfg.min_period_seconds,
            "default_allowance_periods": cfg.default_allowance_periods,
            "allowed_mint": cfg.allowed_mint.to_string(),
            "max_withdrawal_amount": cfg.max_withdrawal_amount,
            "max_withdrawal_amount_usdc": config.format_usdc(cfg.max_withdrawal_amount),
            "max_grace_period_seconds": cfg.max_grace_period_seconds,
            "paused": cfg.paused,
            "keeper_fee_bps": cfg.keeper_fee_bps,
            "keeper_fee_pct": config.format_fee_percentage(cfg.keeper_fee_bps),
            "bump": cfg.bump,
        });
        Ok(serde_json::to_string_pretty(&json_output)?)
    } else {
        // Human-readable output with colors
        let pending_auth = cfg
            .pending_authority
            .map_or_else(|| "None".to_string(), |p| p.to_string());

        let paused_status = if cfg.paused {
            Theme::warning("Yes")
        } else {
            Theme::active("No")
        };

        let mut output = String::new();
        writeln!(&mut output, "{}", Theme::header("Global Configuration"))?;
        writeln!(&mut output, "{}", Theme::dim("===================="))?;
        writeln!(&mut output, "{:<26} {}", Theme::info("Platform Authority:"), Theme::highlight(&cfg.platform_authority.to_string()))?;
        writeln!(&mut output, "{:<26} {}", Theme::info("Pending Authority:"), Theme::dim(&pending_auth))?;
        writeln!(&mut output, "{:<26} {} bps ({}%)", Theme::info("Max Platform Fee:"), cfg.max_platform_fee_bps, Theme::value(&config.format_fee_percentage(cfg.max_platform_fee_bps).to_string()))?;
        writeln!(&mut output, "{:<26} {} bps ({}%)", Theme::info("Min Platform Fee:"), cfg.min_platform_fee_bps, Theme::value(&config.format_fee_percentage(cfg.min_platform_fee_bps).to_string()))?;
        writeln!(&mut output, "{:<26} {} seconds ({} days)", Theme::info("Min Period:"), cfg.min_period_seconds, cfg.min_period_seconds / 86400)?;
        writeln!(&mut output, "{:<26} {}", Theme::info("Default Allowance Periods:"), Theme::value(&cfg.default_allowance_periods.to_string()))?;
        writeln!(&mut output, "{:<26} {}", Theme::info("Allowed Mint (USDC):"), Theme::dim(&cfg.allowed_mint.to_string()))?;
        writeln!(&mut output, "{:<26} {} micro-units ({} USDC)", Theme::info("Max Withdrawal Amount:"), cfg.max_withdrawal_amount, Theme::value(&format!("{:.6}", config.format_usdc(cfg.max_withdrawal_amount))))?;
        writeln!(&mut output, "{:<26} {} seconds ({} days)", Theme::info("Max Grace Period:"), cfg.max_grace_period_seconds, cfg.max_grace_period_seconds / 86400)?;
        writeln!(&mut output, "{:<26} {}", Theme::info("Paused:"), paused_status)?;
        writeln!(&mut output, "{:<26} {} bps ({}%)", Theme::info("Keeper Fee:"), cfg.keeper_fee_bps, Theme::value(&config.format_fee_percentage(cfg.keeper_fee_bps).to_string()))?;
        write!(&mut output, "{:<26} {}", Theme::info("Bump:"), Theme::dim(&cfg.bump.to_string()))?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let request = ShowConfigRequest {
            output_format: "human",
        };
        assert_eq!(request.output_format, "human");

        let request_json = ShowConfigRequest {
            output_format: "json",
        };
        assert_eq!(request_json.output_format, "json");
    }
}
