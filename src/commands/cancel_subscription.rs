//! Cancel subscription command implementation

use crate::config::TallyCliConfig;
use anyhow::{anyhow, Context, Result};
use std::str::FromStr;
use tally_sdk::{
    ata::TokenProgram,
    load_keypair,
    program_types::Subscription,
    transaction_builder,
    AnchorDeserialize,
    SimpleTallyClient,
};
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tally_sdk::solana_sdk::transaction::Transaction;
use tracing::info;

/// Request structure for cancel subscription command
pub struct CancelSubscriptionRequest<'a> {
    pub subscription: &'a str,
    pub subscriber: &'a str,
}

/// Execute the cancel subscription command
///
/// # Errors
///
/// Returns an error if:
/// - The subscription or subscriber address cannot be parsed
/// - The subscriber keypair cannot be loaded or doesn't match the provided address
/// - The subscription account cannot be fetched or deserialized
/// - The subscriber doesn't match the subscription's subscriber
/// - The transaction fails to be sent or confirmed
#[allow(clippy::cognitive_complexity)] // Complex validation logic for subscription cancellation
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &CancelSubscriptionRequest<'_>,
    subscriber_path: Option<&str>,
    config: &TallyCliConfig,
) -> Result<String> {
    info!("Starting subscription cancellation");

    // Parse subscription address
    let subscription = Pubkey::from_str(request.subscription)
        .with_context(|| format!("Failed to parse subscription address '{}'", request.subscription))?;
    info!("Canceling subscription: {}", subscription);

    // Parse subscriber address
    let subscriber_pubkey = Pubkey::from_str(request.subscriber)
        .with_context(|| format!("Failed to parse subscriber address '{}'", request.subscriber))?;
    info!("Subscriber: {}", subscriber_pubkey);

    // Load subscriber keypair
    let subscriber = load_keypair(subscriber_path)
        .context("Failed to load subscriber keypair")?;

    // Verify subscriber keypair matches provided address
    if subscriber.pubkey() != subscriber_pubkey {
        return Err(anyhow!(
            "Subscriber keypair public key ({}) does not match provided subscriber address ({})",
            subscriber.pubkey(),
            subscriber_pubkey
        ));
    }

    // Fetch subscription account to get plan information
    let account = tally_client
        .rpc()
        .get_account(&subscription)
        .with_context(|| format!("Failed to fetch subscription account {subscription}"))?;

    // Deserialize the subscription account
    let subscription_account: Subscription =
        AnchorDeserialize::deserialize(&mut &account.data[8..]) // Skip discriminator
            .context("Failed to deserialize subscription account")?;

    // Verify subscriber matches
    if subscription_account.subscriber != subscriber_pubkey {
        return Err(anyhow!(
            "Subscriber mismatch: subscription has {} but provided {}",
            subscription_account.subscriber,
            subscriber_pubkey
        ));
    }

    // Check if subscription is already canceled (inactive)
    if !subscription_account.active {
        return Err(anyhow!(
            "Subscription is already canceled (inactive)"
        ));
    }

    // Fetch plan account to get merchant
    let plan_account = tally_client
        .rpc()
        .get_account(&subscription_account.plan)
        .with_context(|| format!("Failed to fetch plan account {}", subscription_account.plan))?;

    let plan: tally_sdk::program_types::Plan =
        AnchorDeserialize::deserialize(&mut &plan_account.data[8..]) // Skip discriminator
            .context("Failed to deserialize plan account")?;

    // Fetch merchant account to get USDC mint
    let merchant_address = tally_sdk::pda::merchant_address(&plan.merchant)?;
    let merchant = tally_client
        .get_merchant(&merchant_address)
        .context("Failed to fetch merchant account")?
        .ok_or_else(|| anyhow!("Merchant account not found at {merchant_address}"))?;

    // Build and submit cancel subscription instructions using transaction builder
    let instructions = transaction_builder::cancel_subscription()
        .plan(subscription_account.plan)
        .subscriber(subscriber.pubkey())
        .token_program(TokenProgram::Token)
        .build_instructions(&merchant)
        .context("Failed to build cancel subscription instructions")?;

    // Create transaction with both revoke and cancel instructions
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&subscriber.pubkey()));

    let signature = tally_client
        .submit_transaction(&mut transaction, &[&subscriber])
        .context("Failed to submit cancel subscription transaction")?;

    info!("Transaction confirmed: {}", signature);

    // Return success message with transaction details
    let price_usdc = config.format_usdc(plan.price_usdc);
    let period_days = plan.period_secs / 86400;
    let period_hours = (plan.period_secs % 86400) / 3600;

    Ok(format!(
        "Subscription canceled successfully!\n\
        Subscription: {subscription}\n\
        Plan: {}\n\
        Subscriber: {subscriber_pubkey}\n\
        Price: {price_usdc:.6} USDC\n\
        Period: {} days {} hours\n\
        Transaction signature: {signature}\n\n\
        Note: You can reclaim the rent (~0.001 SOL) by running:\n\
        tally-cli close-subscription --subscription {subscription} --subscriber {subscriber_pubkey}",
        subscription_account.plan,
        period_days,
        period_hours
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_request() {
        let request = CancelSubscriptionRequest {
            subscription: "11111111111111111111111111111111",
            subscriber: "11111111111111111111111111111111",
        };
        assert_eq!(request.subscription, "11111111111111111111111111111111");
        assert_eq!(request.subscriber, "11111111111111111111111111111111");
    }
}
