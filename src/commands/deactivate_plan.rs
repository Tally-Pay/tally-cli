//! Deactivate plan command implementation

use anyhow::{anyhow, Result};
use std::str::FromStr;
use tally_sdk::{
    load_keypair, pda,
    SimpleTallyClient,
};
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tracing::info;

/// Execute the deactivate plan command
///
/// # Errors
/// Returns error if plan deactivation fails due to invalid parameters, network issues, or Solana program errors
#[allow(clippy::cognitive_complexity)] // Complex validation logic for plan deactivation
pub async fn execute(
    tally_client: &SimpleTallyClient,
    plan_str: &str,
    authority_path: Option<&str>,
) -> Result<String> {
    info!("Starting plan deactivation");

    // Parse plan PDA address
    let plan_pda = Pubkey::from_str(plan_str)
        .map_err(|e| anyhow!("Invalid plan PDA address '{plan_str}': {e}"))?;
    info!("Using plan PDA: {}", plan_pda);

    // Load authority keypair
    let authority = load_keypair(authority_path)?;
    info!("Using authority: {}", authority.pubkey());

    // Fetch and validate plan account using tally-sdk
    let plan = tally_client
        .get_plan(&plan_pda)?
        .ok_or_else(|| anyhow!("Plan account does not exist at address: {plan_pda}"))?;

    // Validate authority matches merchant authority by computing expected merchant PDA
    let authority_pubkey = Pubkey::from(authority.pubkey().to_bytes());
    let expected_merchant_pda = pda::merchant_address(&authority_pubkey)?;
    if plan.merchant != expected_merchant_pda {
        return Err(anyhow!(
            "Authority mismatch: this authority ({}) does not own the merchant ({}) for this plan. Expected merchant: {}",
            authority.pubkey(),
            plan.merchant,
            expected_merchant_pda
        ));
    }

    // Check if plan is already deactivated
    if !plan.active {
        return Err(anyhow!(
            "Plan '{}' is already deactivated",
            plan.plan_id_str()
        ));
    }

    info!(
        "Plan '{}' is currently active, proceeding with deactivation",
        plan.plan_id_str()
    );

    // Fetch merchant account (required for builder validation)
    let merchant = tally_client
        .get_merchant(&plan.merchant)?
        .ok_or_else(|| anyhow!("Merchant account not found at address: {}", plan.merchant))?;

    // Build update_plan instruction to set active = false
    info!("Building update_plan instruction to deactivate plan");

    let update_args = tally_sdk::program_types::UpdatePlanArgs {
        name: None,
        active: Some(false),
        price_usdc: None,
        period_secs: None,
        grace_secs: None,
    };

    let instruction = tally_sdk::UpdatePlanBuilder::new()
        .authority(authority.pubkey())
        .payer(authority.pubkey())
        .plan_key(plan_pda)
        .update_args(update_args)
        .program_id(tally_client.program_id())
        .build_instruction(&merchant)?;

    info!("Submitting transaction...");

    // Create and submit transaction
    let mut transaction = tally_sdk::solana_sdk::transaction::Transaction::new_with_payer(
        &[instruction],
        Some(&authority.pubkey()),
    );

    let signature = tally_client.submit_transaction(&mut transaction, &[&authority])?;

    info!("Transaction confirmed: {}", signature);

    // Return success message
    Ok(format!(
        "Plan deactivated successfully!\nPlan PDA: {}\nPlan ID: {}\nNew Active Status: false\nTransaction: {}",
        plan_pda,
        plan.plan_id_str(),
        signature
    ))
}
