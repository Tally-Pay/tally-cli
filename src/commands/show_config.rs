//! Show global configuration account details

use anyhow::{Context, Result};
use tally_sdk::SimpleTallyClient;
use crate::config::TallyCliConfig;

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
        // Human-readable output
        let pending_auth = cfg.pending_authority
            .map_or_else(|| "None".to_string(), |p| p.to_string());

        Ok(format!(
            "Global Configuration
====================
Platform Authority:        {}
Pending Authority:         {}
Max Platform Fee:          {} ({})
Min Platform Fee:          {} ({})
Min Period:                {} seconds ({} days)
Default Allowance Periods: {}
Allowed Mint (USDC):       {}
Max Withdrawal Amount:     {} micro-units ({})
Max Grace Period:          {} seconds ({} days)
Paused:                    {}
Keeper Fee:                {} ({})
Bump:                      {}
",
            cfg.platform_authority,
            pending_auth,
            cfg.max_platform_fee_bps,
            config.format_fee_percentage(cfg.max_platform_fee_bps),
            cfg.min_platform_fee_bps,
            config.format_fee_percentage(cfg.min_platform_fee_bps),
            cfg.min_period_seconds,
            cfg.min_period_seconds / 86400,
            cfg.default_allowance_periods,
            cfg.allowed_mint,
            cfg.max_withdrawal_amount,
            config.format_usdc(cfg.max_withdrawal_amount),
            cfg.max_grace_period_seconds,
            cfg.max_grace_period_seconds / 86400,
            cfg.paused,
            cfg.keeper_fee_bps,
            config.format_fee_percentage(cfg.keeper_fee_bps),
            cfg.bump,
        ))
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
