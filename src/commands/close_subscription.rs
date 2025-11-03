//! Close subscription command implementation

use crate::config::TallyCliConfig;
use anyhow::{anyhow, Context, Result};
use std::str::FromStr;
use tally_sdk::{load_keypair, AnchorDeserialize, SimpleTallyClient};
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tracing::info;

/// Request parameters for closing a subscription
pub struct CloseSubscriptionRequest<'a> {
    pub subscription: &'a str,
    pub subscriber: &'a str,
}

/// Execute the close subscription command
///
/// # Errors
///
/// Returns an error if:
/// - The subscription or subscriber address cannot be parsed
/// - The subscriber keypair cannot be loaded
/// - The subscription account doesn't exist
/// - The subscription is not in Canceled status
/// - The close transaction fails to be sent or confirmed
#[allow(clippy::cognitive_complexity)] // Complex validation logic for closing subscriptions
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &CloseSubscriptionRequest<'_>,
    subscriber_path: Option<&str>,
    _config: &TallyCliConfig,
) -> Result<String> {
    const SUBSCRIPTION_RENT_LAMPORTS: u64 = 997_920; // ~0.00099792 SOL

    info!("Starting subscription close operation");

    // Parse subscription address
    let subscription = Pubkey::from_str(request.subscription)
        .with_context(|| format!("Failed to parse subscription address '{}'", request.subscription))?;
    info!("Closing subscription: {}", subscription);

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
    let subscription_account: tally_sdk::program_types::Subscription =
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

    // Build and submit close subscription instruction using transaction builder
    let instruction = tally_sdk::transaction_builder::close_subscription()
        .plan(subscription_account.plan)
        .subscriber(subscriber.pubkey())
        .build_instruction()
        .context("Failed to build close subscription instruction")?;

    let signature = tally_client
        .submit_instruction(instruction, &[&subscriber])
        .with_context(|| "Failed to close subscription - check RPC connection and subscription state")?;

    info!("Transaction confirmed: {}", signature);

    // Format rent reclaimed amount (approximate)
    let rent_sol = f64::from(u32::try_from(SUBSCRIPTION_RENT_LAMPORTS).unwrap_or(0)) / 1_000_000_000.0;

    Ok(format!(
        "Subscription closed successfully!\n\
        Subscription: {subscription}\n\
        Subscriber: {}\n\
        Transaction signature: {signature}\n\
        Reclaimed rent: ~{rent_sol:.8} SOL",
        subscriber.pubkey()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_request() {
        let request = CloseSubscriptionRequest {
            subscription: "11111111111111111111111111111111",
            subscriber: "11111111111111111111111111111112",
        };

        assert_eq!(request.subscription, "11111111111111111111111111111111");
        assert_eq!(request.subscriber, "11111111111111111111111111111112");
    }
}
