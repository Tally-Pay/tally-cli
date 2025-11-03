//! Renew subscription command implementation

use crate::config::TallyCliConfig;
use anyhow::{anyhow, Context, Result};
use std::str::FromStr;
use tally_sdk::{
    ata::{get_associated_token_address_with_program, TokenProgram},
    get_usdc_mint, load_keypair, pda, AnchorDeserialize, SimpleTallyClient,
};
use tally_sdk::program_types::Subscription;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tracing::info;

/// Execute the renew subscription command
///
/// # Errors
///
/// Returns an error if:
/// - The keeper keypair cannot be loaded
/// - The subscription PDA is invalid or cannot be parsed
/// - The subscription account does not exist
/// - The plan account does not exist
/// - The merchant account does not exist
/// - The USDC mint validation fails
/// - The transaction fails to be sent or confirmed
#[allow(clippy::cognitive_complexity)] // Complex validation logic for subscription renewal
pub async fn execute(
    tally_client: &SimpleTallyClient,
    subscription_str: &str,
    keeper_path: Option<&str>,
    usdc_mint_str: Option<&str>,
    config: &TallyCliConfig,
) -> Result<String> {
    info!("Renewing subscription");

    // Load keeper keypair
    let keeper = load_keypair(keeper_path).context("Failed to load keeper keypair")?;
    let keeper_pubkey = keeper.pubkey();
    info!("Using keeper: {}", keeper_pubkey);

    // Parse subscription PDA
    let subscription = Pubkey::from_str(subscription_str)
        .with_context(|| format!("Failed to parse subscription PDA '{subscription_str}'"))?;
    info!("Using subscription: {}", subscription);

    // Parse USDC mint using tally-sdk
    let usdc_mint = get_usdc_mint(usdc_mint_str).context("Failed to parse USDC mint")?;
    info!("Using USDC mint: {}", usdc_mint);

    // Fetch subscription account to get plan and subscriber
    info!("Fetching subscription account: {}", subscription);
    let subscription_account = tally_client
        .rpc()
        .get_account(&subscription)
        .with_context(|| format!("Failed to fetch subscription account {subscription}"))?;

    // Deserialize the subscription account
    let subscription_data: Subscription =
        AnchorDeserialize::deserialize(&mut &subscription_account.data[8..]) // Skip discriminator
            .context("Failed to deserialize subscription account")?;

    info!("Subscription plan: {}", subscription_data.plan);
    info!("Subscription subscriber: {}", subscription_data.subscriber);

    // Fetch plan account to get merchant reference
    info!("Fetching plan account: {}", subscription_data.plan);
    let plan_account = tally_client
        .rpc()
        .get_account(&subscription_data.plan)
        .with_context(|| format!("Failed to fetch plan account {}", subscription_data.plan))?;

    let plan_data: tally_sdk::program_types::Plan =
        AnchorDeserialize::deserialize(&mut &plan_account.data[8..]) // Skip discriminator
            .context("Failed to deserialize plan account")?;

    info!("Plan merchant: {}", plan_data.merchant);

    // Fetch merchant account to get treasury and config
    let merchant_address = plan_data.merchant;
    info!("Fetching merchant account: {}", merchant_address);
    let merchant_data = tally_client
        .get_merchant(&merchant_address)
        .context("Failed to fetch merchant account")?
        .ok_or_else(|| anyhow!("Merchant account not found at {merchant_address}"))?;
    info!("Merchant authority: {}", merchant_data.authority);

    // Fetch config to get platform authority
    let config_pda = pda::config_address().context("Failed to compute config PDA")?;
    let config_account = tally_client
        .rpc()
        .get_account_data(&config_pda)
        .context("Failed to fetch config account")?;

    // Deserialize config account (skip 8-byte discriminator)
    let config_data: tally_sdk::program_types::Config =
        AnchorDeserialize::deserialize(&mut &config_account[8..])
            .context("Failed to deserialize config account")?;

    let platform_treasury_ata = get_associated_token_address_with_program(
        &config_data.platform_authority,
        &usdc_mint,
        TokenProgram::Token,
    )
    .context("Failed to compute platform treasury ATA")?;
    info!("Platform treasury ATA: {}", platform_treasury_ata);

    // Compute keeper ATA for receiving keeper fee
    let keeper_ata = get_associated_token_address_with_program(
        &keeper_pubkey,
        &usdc_mint,
        TokenProgram::Token,
    )
    .context("Failed to compute keeper ATA")?;
    info!("Keeper ATA: {}", keeper_ata);

    // Build renew subscription instruction using transaction builder
    let instruction = tally_sdk::transaction_builder::renew_subscription()
        .plan(subscription_data.plan)
        .subscriber(subscription_data.subscriber)
        .keeper(keeper_pubkey)
        .keeper_ata(keeper_ata)
        .build_instruction(&merchant_data, &plan_data, &platform_treasury_ata)
        .context("Failed to build renew subscription instruction")?;

    info!("Built renew subscription instruction");

    // Submit transaction
    let signature = tally_client
        .submit_instruction(instruction, &[&keeper])
        .context("Failed to renew subscription")?;

    info!("Transaction confirmed: {}", signature);

    // Calculate keeper fee (0.5% = 50 bps)
    let keeper_fee_bps = config_data.keeper_fee_bps;
    let keeper_fee = plan_data
        .price_usdc
        .checked_mul(u64::from(keeper_fee_bps))
        .and_then(|v| v.checked_div(10000))
        .context("Arithmetic overflow calculating keeper fee")?;

    // Return success message with renewal details
    let price_usdc = config.format_usdc(plan_data.price_usdc);
    let keeper_fee_usdc = config.format_usdc(keeper_fee);

    Ok(format!(
        "Subscription renewed successfully!\n\
        Subscription: {subscription}\n\
        Transaction signature: {signature}\n\
        Plan: {}\n\
        Subscriber: {}\n\
        Price: {price_usdc:.6} USDC\n\
        Keeper: {keeper_pubkey}\n\
        Keeper fee: {keeper_fee_usdc:.6} USDC",
        subscription_data.plan, subscription_data.subscriber
    ))
}
