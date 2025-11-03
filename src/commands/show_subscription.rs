//! Show subscription account details

use anyhow::{Context, Result};
use tally_sdk::SimpleTallyClient;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::config::TallyCliConfig;
use crate::utils::formatting;

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
        // Human-readable output
        let status = if subscription.active {
            if subscription.in_trial {
                "Active (In Trial)"
            } else {
                "Active"
            }
        } else {
            "Canceled"
        };

        let trial_info = subscription.trial_ends_at.map_or_else(
            String::new,
            |trial_ends| format!(
                "Trial Ends:            {} ({})\n",
                trial_ends,
                formatting::format_timestamp(trial_ends)
            )
        );

        Ok(format!(
            "Subscription Account Details
============================
Subscription PDA:      {}
Plan:                  {}
Subscriber:            {}
Status:                {}
Active:                {}
In Trial:              {}
{}Next Renewal:         {} ({})
Created:               {} ({})
Last Amount:           {} micro-units ({})
Last Renewed:          {} ({})
Renewals:              {}
Bump:                  {}
",
            request.subscription,
            subscription.plan,
            subscription.subscriber,
            status,
            subscription.active,
            subscription.in_trial,
            trial_info,
            subscription.next_renewal_ts,
            formatting::format_timestamp(subscription.next_renewal_ts),
            subscription.created_ts,
            formatting::format_timestamp(subscription.created_ts),
            subscription.last_amount,
            config.format_usdc(subscription.last_amount),
            subscription.last_renewed_ts,
            formatting::format_timestamp(subscription.last_renewed_ts),
            subscription.renewals,
            subscription.bump,
        ))
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
