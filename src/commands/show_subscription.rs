//! Show subscription account details

use crate::config::TallyCliConfig;
use crate::utils::colors::Theme;
use crate::utils::formatting;
use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::SimpleTallyClient;

/// Request to show subscription details
pub struct ShowSubscriptionRequest<'a> {
    /// Subscription PDA address
    pub subscription: &'a str,
    /// Output format
    pub output_format: &'a str,
}

/// Execute the show-subscription command
///
/// # Arguments
/// * `tally_client` - The Tally SDK client
/// * `request` - The show subscription request parameters
/// * `config` - CLI configuration
///
/// # Returns
/// * `Ok(String)` - Formatted subscription details
///
/// # Errors
/// Returns an error if:
/// * Subscription public key cannot be parsed
/// * Failed to fetch subscription account from RPC
/// * Subscription account not found
/// * JSON serialization fails
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &ShowSubscriptionRequest<'_>,
    config: &TallyCliConfig,
) -> Result<String> {
    // Parse subscription address
    let subscription_address = Pubkey::from_str(request.subscription)
        .context("Failed to parse subscription public key")?;

    // Fetch subscription account
    let subscription = tally_client
        .get_subscription(&subscription_address)
        .context("Failed to fetch subscription account - check RPC connection and account state")?
        .context("Subscription account not found")?;

    // Format output based on requested format
    if request.output_format == "json" {
        let json_output = serde_json::json!({
            "subscription": request.subscription,
            "plan": subscription.plan.to_string(),
            "subscriber": subscription.subscriber.to_string(),
            "next_renewal_ts": subscription.next_renewal_ts,
            "next_renewal_human": formatting::format_timestamp(subscription.next_renewal_ts),
            "active": subscription.active,
            "renewals": subscription.renewals,
            "created_ts": subscription.created_ts,
            "created_human": formatting::format_timestamp(subscription.created_ts),
            "last_amount": subscription.last_amount,
            "last_amount_usdc": config.format_usdc(subscription.last_amount),
            "last_renewed_ts": subscription.last_renewed_ts,
            "last_renewed_human": formatting::format_timestamp(subscription.last_renewed_ts),
            "trial_ends_at": subscription.trial_ends_at,
            "trial_ends_at_human": subscription.trial_ends_at.map(formatting::format_timestamp),
            "in_trial": subscription.in_trial,
            "bump": subscription.bump,
        });
        Ok(serde_json::to_string_pretty(&json_output)?)
    } else {
        // Human-readable output with colors
        let status_text = if subscription.active {
            if subscription.in_trial {
                Theme::active("Active (In Trial)")
            } else {
                Theme::active("Active")
            }
        } else {
            Theme::inactive("Canceled")
        };

        let active_status = if subscription.active {
            Theme::active("Yes")
        } else {
            Theme::inactive("No")
        };

        let trial_status = if subscription.in_trial {
            Theme::warning("Yes")
        } else {
            Theme::dim("No")
        };

        let mut output = String::new();
        writeln!(&mut output, "{}", Theme::header("Subscription Account Details"))?;
        writeln!(&mut output, "{}", Theme::dim("============================"))?;
        writeln!(&mut output, "{:<22} {}", Theme::info("Subscription PDA:"), Theme::highlight(request.subscription))?;
        writeln!(&mut output, "{:<22} {}", Theme::info("Plan:"), Theme::dim(&subscription.plan.to_string()))?;
        writeln!(&mut output, "{:<22} {}", Theme::info("Subscriber:"), Theme::value(&subscription.subscriber.to_string()))?;
        writeln!(&mut output, "{:<22} {}", Theme::info("Status:"), status_text)?;
        writeln!(&mut output, "{:<22} {}", Theme::info("Active:"), active_status)?;
        writeln!(&mut output, "{:<22} {}", Theme::info("In Trial:"), trial_status)?;

        // Trial ends info if applicable
        if let Some(trial_ends) = subscription.trial_ends_at {
            writeln!(&mut output, "{:<22} {} ({})",
                Theme::info("Trial Ends:"),
                trial_ends,
                Theme::dim(&formatting::format_timestamp(trial_ends)))?;
        }

        writeln!(&mut output, "{:<22} {} ({})",
            Theme::info("Next Renewal:"),
            subscription.next_renewal_ts,
            Theme::dim(&formatting::format_timestamp(subscription.next_renewal_ts)))?;
        writeln!(&mut output, "{:<22} {} ({})",
            Theme::info("Created:"),
            subscription.created_ts,
            Theme::dim(&formatting::format_timestamp(subscription.created_ts)))?;
        writeln!(&mut output, "{:<22} {} micro-units ({} USDC)",
            Theme::info("Last Amount:"),
            subscription.last_amount,
            Theme::value(&format!("{:.6}", config.format_usdc(subscription.last_amount))))?;
        writeln!(&mut output, "{:<22} {} ({})",
            Theme::info("Last Renewed:"),
            subscription.last_renewed_ts,
            Theme::dim(&formatting::format_timestamp(subscription.last_renewed_ts)))?;
        writeln!(&mut output, "{:<22} {}", Theme::info("Renewals:"), Theme::value(&subscription.renewals.to_string()))?;
        write!(&mut output, "{:<22} {}", Theme::info("Bump:"), Theme::dim(&subscription.bump.to_string()))?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let request = ShowSubscriptionRequest {
            subscription: "11111111111111111111111111111111",
            output_format: "human",
        };
        assert_eq!(request.subscription, "11111111111111111111111111111111");
        assert_eq!(request.output_format, "human");
    }
}
