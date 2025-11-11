//! Show payment agreement account details

use crate::config::TallyCliConfig;
use crate::utils::colors::Theme;
use crate::utils::formatting;
use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::SimpleTallyClient;

/// Request to show payment agreement details
pub struct ShowAgreementRequest<'a> {
    /// Payment Agreement PDA address
    pub agreement: &'a str,
    /// Output format
    pub output_format: &'a str,
}

/// Execute the show-agreement command
///
/// # Arguments
/// * `tally_client` - The Tally SDK client
/// * `request` - The show agreement request parameters
/// * `config` - CLI configuration
///
/// # Returns
/// * `Ok(String)` - Formatted agreement details
///
/// # Errors
/// Returns an error if:
/// * Agreement public key cannot be parsed
/// * Failed to fetch agreement account from RPC
/// * Agreement account not found
/// * JSON serialization fails
///
/// # Panics
/// This function does not panic under normal operation.
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &ShowAgreementRequest<'_>,
    _config: &TallyCliConfig,
) -> Result<String> {
    // Parse agreement address
    let agreement_address = Pubkey::from_str(request.agreement)
        .context("Failed to parse payment agreement public key")?;

    // Fetch payment agreement account
    let agreement = tally_client
        .get_payment_agreement(&agreement_address)
        .context(
            "Failed to fetch payment agreement account - check RPC connection and account state",
        )?
        .context("Payment agreement account not found")?;

    // Create type-safe amount
    let last_amount = tally_sdk::UsdcAmount::from_microlamports(agreement.last_amount);

    // Format output based on requested format
    if request.output_format == "json" {
        let json_output = serde_json::json!({
            "agreement": request.agreement,
            "payment_terms": agreement.payment_terms.to_string(),
            "payer": agreement.payer.to_string(),
            "next_payment_ts": agreement.next_payment_ts,
            "next_payment_human": formatting::format_timestamp(agreement.next_payment_ts),
            "active": agreement.active,
            "payment_count": agreement.payment_count,
            "created_ts": agreement.created_ts,
            "created_human": formatting::format_timestamp(agreement.created_ts),
            "last_amount_microlamports": last_amount.microlamports(),
            "last_amount_usdc": last_amount.usdc(),
            "last_amount_display": last_amount.to_string(),
            "last_payment_ts": agreement.last_payment_ts,
            "last_payment_human": formatting::format_timestamp(agreement.last_payment_ts),
            "bump": agreement.bump,
        });
        Ok(serde_json::to_string_pretty(&json_output)?)
    } else {
        // Human-readable output with colors
        let status_text = if agreement.active {
            Theme::active("Active")
        } else {
            Theme::inactive("Paused")
        };

        let active_status = if agreement.active {
            Theme::active("Yes")
        } else {
            Theme::inactive("No")
        };

        let mut output = String::new();
        writeln!(
            &mut output,
            "{}",
            Theme::header("Payment Agreement Details")
        )?;
        writeln!(&mut output, "{}", Theme::dim("==========================="))?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Agreement PDA:"),
            Theme::highlight(request.agreement)
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Payment Terms:"),
            Theme::dim(&agreement.payment_terms.to_string())
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Payer:"),
            Theme::value(&agreement.payer.to_string())
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Status:"),
            status_text
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Active:"),
            active_status
        )?;
        writeln!(
            &mut output,
            "{:<22} {} ({})",
            Theme::info("Next Payment:"),
            agreement.next_payment_ts,
            Theme::dim(&formatting::format_timestamp(agreement.next_payment_ts))
        )?;
        writeln!(
            &mut output,
            "{:<22} {} ({})",
            Theme::info("Created:"),
            agreement.created_ts,
            Theme::dim(&formatting::format_timestamp(agreement.created_ts))
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Last Amount:"),
            Theme::value(&last_amount.to_string())
        )?;
        writeln!(
            &mut output,
            "{:<22} {} ({})",
            Theme::info("Last Payment:"),
            agreement.last_payment_ts,
            Theme::dim(&formatting::format_timestamp(agreement.last_payment_ts))
        )?;
        writeln!(
            &mut output,
            "{:<22} {}",
            Theme::info("Payment Count:"),
            Theme::value(&agreement.payment_count.to_string())
        )?;
        write!(
            &mut output,
            "{:<22} {}",
            Theme::info("Bump:"),
            Theme::dim(&agreement.bump.to_string())
        )?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let request = ShowAgreementRequest {
            agreement: "11111111111111111111111111111111",
            output_format: "human",
        };
        assert_eq!(request.agreement, "11111111111111111111111111111111");
        assert_eq!(request.output_format, "human");
    }
}
