//! Start subscription command implementation

use crate::config::TallyCliConfig;
use anyhow::{anyhow, Context, Result};
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::{
    ata::TokenProgram, get_usdc_mint, load_keypair, pda, transaction_builder::start_subscription,
    AnchorDeserialize, SimpleTallyClient,
};
use tally_sdk::solana_sdk::message::Message;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tally_sdk::solana_sdk::transaction::{Transaction, VersionedTransaction};
use tracing::info;

/// Request structure for starting a subscription
pub struct StartSubscriptionRequest<'a> {
    pub plan: &'a str,
    #[allow(dead_code)] // Used indirectly via parsing in main
    pub subscriber: &'a str,
    pub allowance_periods: Option<u8>,
    pub trial_duration_secs: Option<u64>,
}

/// Execute the start subscription command
///
/// # Errors
/// Returns error if subscription start fails due to invalid parameters, network issues, or Solana program errors
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &StartSubscriptionRequest<'_>,
    subscriber_keypair_path: Option<&str>,
    usdc_mint_str: Option<&str>,
    config: &TallyCliConfig,
) -> Result<String> {
    info!("Starting subscription to plan");

    // Load subscriber keypair
    let subscriber_keypair = load_keypair(subscriber_keypair_path)
        .context("Failed to load subscriber keypair")?;
    let subscriber_pubkey = subscriber_keypair.pubkey();
    info!("Using subscriber: {}", subscriber_pubkey);

    // Parse plan PDA
    let plan_pda = Pubkey::from_str(request.plan)
        .context("Failed to parse plan public key")?;
    info!("Plan PDA: {}", plan_pda);

    // Get USDC mint
    let usdc_mint = get_usdc_mint(usdc_mint_str)
        .context("Failed to get USDC mint")?;
    info!("Using USDC mint: {}", usdc_mint);

    // Fetch plan data
    let plan_data = tally_client
        .get_plan(&plan_pda)
        .context("Failed to fetch plan data - ensure plan exists")?
        .ok_or_else(|| anyhow!("Plan not found at address: {plan_pda}"))?;

    // Convert name bytes to string
    let plan_name = String::from_utf8_lossy(&plan_data.name)
        .trim_end_matches('\0')
        .to_string();

    info!(
        "Plan details: {} - {} USDC every {} seconds",
        plan_name,
        config.format_usdc(plan_data.price_usdc),
        plan_data.period_secs
    );

    // Fetch merchant data
    let merchant_pda = plan_data.merchant;
    let merchant_data = tally_client
        .get_merchant(&merchant_pda)
        .context("Failed to fetch merchant data")?
        .ok_or_else(|| anyhow!("Merchant not found at address: {merchant_pda}"))?;
    info!("Merchant: {}", plan_data.merchant);

    // Fetch config to get platform authority
    let config_pda = pda::config_address()?;
    let config_account = tally_client
        .rpc()
        .get_account_data(&config_pda)
        .context("Failed to fetch config account")?;

    // Deserialize config account (skip 8-byte discriminator)
    let config_data: tally_sdk::program_types::Config =
        AnchorDeserialize::deserialize(&mut &config_account[8..])
            .context("Failed to deserialize config account")?;
    let platform_authority = config_data.platform_authority;

    let platform_treasury_ata = tally_sdk::ata::get_associated_token_address_with_program(
        &platform_authority,
        &usdc_mint,
        TokenProgram::Token,
    )
    .context("Failed to compute platform treasury ATA")?;

    // Build instructions
    let mut builder = start_subscription()
        .plan(plan_pda)
        .subscriber(subscriber_pubkey)
        .payer(subscriber_pubkey)
        .program_id(tally_client.program_id());

    if let Some(periods) = request.allowance_periods {
        builder = builder.allowance_periods(periods);
    }

    if let Some(trial_secs) = request.trial_duration_secs {
        builder = builder.trial_duration_secs(trial_secs);
    }

    let instructions = builder
        .build_instructions(&merchant_data, &plan_data, &platform_treasury_ata)
        .context("Failed to build start subscription instructions")?;

    info!("Built {} instructions for subscription start", instructions.len());

    // Submit transaction with multiple instructions
    let recent_blockhash = tally_client
        .get_latest_blockhash()
        .context("Failed to get latest blockhash")?;

    // Build legacy transaction
    let message = Message::new_with_blockhash(
        &instructions,
        Some(&subscriber_pubkey),
        &recent_blockhash,
    );
    let transaction = Transaction::new(&[&subscriber_keypair], message, recent_blockhash);
    let versioned_transaction = VersionedTransaction::from(transaction);

    let signature = tally_client
        .send_and_confirm_transaction(&versioned_transaction)
        .context("Failed to submit start subscription transaction")?;

    info!("Transaction confirmed: {}", signature);

    // Compute subscription PDA for output
    let subscription_pda = pda::subscription_address(&plan_pda, &subscriber_pubkey)?;

    // Calculate allowance amount
    let allowance_periods = request.allowance_periods.unwrap_or(3);
    let allowance_amount = plan_data
        .price_usdc
        .checked_mul(u64::from(allowance_periods))
        .ok_or_else(|| anyhow!("Arithmetic overflow calculating allowance amount"))?;

    // Return success message
    let price_usdc = config.format_usdc(plan_data.price_usdc);
    let allowance_usdc = config.format_usdc(allowance_amount);

    let mut output = format!(
        "Subscription started successfully!\n\
        Subscription PDA: {subscription_pda}\n\
        Plan: {} ({})\n\
        Price: {price_usdc:.6} USDC per period\n\
        Period: {} seconds\n\
        Subscriber: {subscriber_pubkey}\n\
        Allowance: {allowance_usdc:.6} USDC ({allowance_periods}× price)\n\
        Transaction signature: {signature}",
        plan_name,
        plan_pda,
        plan_data.period_secs
    );

    if let Some(trial_secs) = request.trial_duration_secs {
        write!(&mut output, "\nTrial period: {trial_secs} seconds")
            .expect("String formatting should not fail");
    }

    Ok(output)
}
