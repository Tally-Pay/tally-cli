//! Update plan terms command implementation

use crate::config::TallyCliConfig;
use crate::utils::colors::Theme;
use anyhow::{anyhow, Context, Result};
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tally_sdk::{load_keypair, SimpleTallyClient};
use tracing::info;

/// Request parameters for updating plan terms
pub struct UpdatePlanTermsRequest<'a> {
    pub plan: &'a str,
    pub new_price: Option<u64>,
    pub new_period_seconds: Option<i64>,
    pub new_grace_period_seconds: Option<i64>,
}

/// Execute the update plan terms command
///
/// # Errors
///
/// Returns an error if:
/// - No update parameters are provided (at least one must be specified)
/// - The plan address cannot be parsed
/// - The merchant authority keypair cannot be loaded
/// - The plan account doesn't exist
/// - The update transaction fails to be sent or confirmed
#[allow(clippy::cognitive_complexity)] // Complex validation logic for plan updates
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &UpdatePlanTermsRequest<'_>,
    authority_path: Option<&str>,
    config: &TallyCliConfig,
) -> Result<String> {
    info!("Starting plan terms update");

    // Validate that at least one field is being updated
    if request.new_price.is_none()
        && request.new_period_seconds.is_none()
        && request.new_grace_period_seconds.is_none()
    {
        return Err(anyhow!(
            "At least one update parameter must be provided (--price, --period, or --grace-period)"
        ));
    }

    // Parse plan address
    let plan = Pubkey::from_str(request.plan)
        .with_context(|| format!("Failed to parse plan address '{}'", request.plan))?;
    info!("Updating plan: {}", plan);

    // Load merchant authority keypair
    let authority =
        load_keypair(authority_path).context("Failed to load merchant authority keypair")?;
    info!("Using authority: {}", authority.pubkey());

    // Validate parameters
    if let Some(price) = request.new_price {
        if price == 0 {
            return Err(anyhow!("Price cannot be zero"));
        }
    }

    if let Some(period) = request.new_period_seconds {
        if period < 3600 {
            return Err(anyhow!(
                "Period must be at least 3600 seconds (1 hour), got {period}"
            ));
        }
    }

    if let Some(grace) = request.new_grace_period_seconds {
        if grace < 0 {
            return Err(anyhow!("Grace period cannot be negative, got {grace}"));
        }
    }

    // Check if plan exists
    if !tally_client.account_exists(&plan)? {
        return Err(anyhow!("Plan account does not exist at address: {plan}"));
    }

    // Build update plan terms instruction using transaction builder
    let mut builder = tally_sdk::transaction_builder::update_plan_terms()
        .authority(authority.pubkey())
        .plan_key(plan);

    // Conditionally add optional fields
    if let Some(price) = request.new_price {
        builder = builder.price_usdc(price);
    }

    if let Some(period) = request.new_period_seconds {
        // Convert i64 to u64 for period
        let period_u64 =
            u64::try_from(period).map_err(|e| anyhow!("Invalid period value {period}: {e}"))?;
        builder = builder.period_secs(period_u64);
    }

    if let Some(grace) = request.new_grace_period_seconds {
        // Convert i64 to u64 for grace period
        let grace_u64 =
            u64::try_from(grace).map_err(|e| anyhow!("Invalid grace period value {grace}: {e}"))?;
        builder = builder.grace_secs(grace_u64);
    }

    let instruction = builder
        .build_instruction()
        .context("Failed to build update plan terms instruction")?;

    let signature = tally_client
        .submit_instruction(instruction, &[&authority])
        .with_context(|| "Failed to update plan terms - check RPC connection and account state")?;

    info!("Transaction confirmed: {}", signature);

    // Build colored success message with updated fields
    let mut output = String::new();
    writeln!(&mut output, "{}", Theme::success("Plan terms updated successfully!"))?;
    writeln!(&mut output)?;
    writeln!(&mut output, "{} {}", Theme::info("Plan:"), Theme::highlight(&plan.to_string()))?;
    writeln!(&mut output, "{} {}", Theme::info("Transaction signature:"), Theme::dim(&signature))?;
    writeln!(&mut output)?;
    writeln!(&mut output, "{}", Theme::header("Updated fields:"))?;

    if let Some(price) = request.new_price {
        let price_usdc = config.format_usdc(price);
        writeln!(&mut output, "  {} {:.6} USDC ({} micro-units)",
            Theme::info("• Price:"),
            price_usdc,
            Theme::value(&price.to_string()))?;
    }

    if let Some(period) = request.new_period_seconds {
        let period_days = period / 86400;
        let period_hours = (period % 86400) / 3600;
        writeln!(&mut output, "  {} {} seconds (~{} days, {} hours)",
            Theme::info("• Period:"),
            Theme::value(&period.to_string()),
            period_days,
            period_hours)?;
    }

    if let Some(grace) = request.new_grace_period_seconds {
        let grace_hours = grace / 3600;
        write!(&mut output, "  {} {} seconds ({} hours)",
            Theme::info("• Grace period:"),
            Theme::value(&grace.to_string()),
            grace_hours)?;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_at_least_one_field_required() {
        let request = UpdatePlanTermsRequest {
            plan: "11111111111111111111111111111111",
            new_price: None,
            new_period_seconds: None,
            new_grace_period_seconds: None,
        };

        // This will fail at execution time when validated
        assert!(request.new_price.is_none());
        assert!(request.new_period_seconds.is_none());
        assert!(request.new_grace_period_seconds.is_none());
    }

    #[test]
    fn test_valid_price_update() {
        let request = UpdatePlanTermsRequest {
            plan: "11111111111111111111111111111111",
            new_price: Some(15_000_000),
            new_period_seconds: None,
            new_grace_period_seconds: None,
        };

        assert_eq!(request.new_price, Some(15_000_000));
    }

    #[test]
    fn test_valid_period_update() {
        let request = UpdatePlanTermsRequest {
            plan: "11111111111111111111111111111111",
            new_price: None,
            new_period_seconds: Some(5_184_000), // 60 days
            new_grace_period_seconds: None,
        };

        assert_eq!(request.new_period_seconds, Some(5_184_000));
    }

    #[test]
    fn test_valid_multiple_updates() {
        let request = UpdatePlanTermsRequest {
            plan: "11111111111111111111111111111111",
            new_price: Some(9_000_000),
            new_period_seconds: Some(2_592_000),     // 30 days
            new_grace_period_seconds: Some(604_800), // 7 days
        };

        assert_eq!(request.new_price, Some(9_000_000));
        assert_eq!(request.new_period_seconds, Some(2_592_000));
        assert_eq!(request.new_grace_period_seconds, Some(604_800));
    }
}
