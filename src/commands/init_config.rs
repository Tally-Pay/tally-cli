//! Init config command implementation

use crate::config::TallyCliConfig;
use anyhow::{anyhow, Result};
use std::str::FromStr;
use tally_sdk::{
    load_keypair,
    program_types::InitConfigArgs,
    transaction_builder::init_config,
    SimpleTallyClient,
};
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tracing::info;

/// Execute the init config command
///
/// # Errors
/// Returns error if configuration initialization fails due to invalid parameters, network issues, or Solana program errors
#[allow(clippy::cognitive_complexity, clippy::too_many_arguments)]
pub async fn execute(
    tally_client: &SimpleTallyClient,
    platform_authority_str: &str,
    max_platform_fee_bps: u16,
    min_platform_fee_bps: u16,
    min_period_seconds: u64,
    default_allowance_periods: u8,
    allowed_mint_str: &str,
    max_withdrawal_amount: u64,
    max_grace_period_seconds: u64,
    keeper_fee_bps: u16,
    authority_path: Option<&str>,
    config: &TallyCliConfig,
) -> Result<String> {
    info!("Starting global config initialization");

    // Load authority keypair
    let authority = load_keypair(authority_path)?;
    info!("Using authority: {}", authority.pubkey());

    // Parse platform authority
    let platform_authority = Pubkey::from_str(platform_authority_str).map_err(|e| {
        anyhow!(
            "Invalid platform authority address '{platform_authority_str}': {e}"
        )
    })?;
    info!("Using platform authority: {}", platform_authority);

    // Parse allowed mint
    let allowed_mint = Pubkey::from_str(allowed_mint_str).map_err(|e| {
        anyhow!(
            "Invalid allowed mint address '{allowed_mint_str}': {e}"
        )
    })?;
    info!("Using allowed mint: {}", allowed_mint);

    // Validate parameters
    if max_platform_fee_bps > 10000 {
        return Err(anyhow!(
            "Max platform fee basis points cannot exceed 10000 (100%)"
        ));
    }
    if min_platform_fee_bps > max_platform_fee_bps {
        return Err(anyhow!(
            "Min platform fee ({min_platform_fee_bps} bps) cannot exceed max platform fee ({max_platform_fee_bps} bps)"
        ));
    }
    if keeper_fee_bps > 10000 {
        return Err(anyhow!(
            "Keeper fee basis points cannot exceed 10000 (100%)"
        ));
    }
    if min_period_seconds < 3600 {
        return Err(anyhow!(
            "Minimum period seconds should be at least 3600 (1 hour)"
        ));
    }
    if default_allowance_periods == 0 {
        return Err(anyhow!("Default allowance periods cannot be zero"));
    }
    if max_withdrawal_amount == 0 {
        return Err(anyhow!("Max withdrawal amount cannot be zero"));
    }
    if max_grace_period_seconds == 0 {
        return Err(anyhow!("Max grace period seconds cannot be zero"));
    }

    // Check if config already exists
    let config_pda = tally_sdk::pda::config_address()?;
    if tally_client.account_exists(&config_pda)? {
        return Err(anyhow!(
            "Config account already exists at address: {config_pda}"
        ));
    }

    // Create config args
    let config_args = InitConfigArgs {
        platform_authority,
        max_platform_fee_bps,
        min_platform_fee_bps,
        min_period_seconds,
        default_allowance_periods,
        allowed_mint,
        max_withdrawal_amount,
        max_grace_period_seconds,
        keeper_fee_bps,
    };

    // Build and submit instruction
    let instruction = init_config()
        .authority(authority.pubkey())
        .payer(authority.pubkey())
        .config_args(config_args)
        .build_instruction()?;

    let signature = tally_client.submit_instruction(instruction, &[&authority])?;

    info!("Transaction confirmed: {}", signature);

    // Return success message with config PDA and transaction signature
    let max_fee_pct = config.format_fee_percentage(max_platform_fee_bps);
    let min_fee_pct = config.format_fee_percentage(min_platform_fee_bps);
    let keeper_fee_pct = config.format_fee_percentage(keeper_fee_bps);
    let max_withdrawal_usdc = config.format_usdc(max_withdrawal_amount);

    Ok(format!(
        "Global config initialized successfully!\n\
        Config PDA: {config_pda}\n\
        Transaction signature: {signature}\n\
        Platform authority: {platform_authority}\n\
        Max platform fee: {max_platform_fee_bps} bps ({max_fee_pct:.2}%)\n\
        Min platform fee: {min_platform_fee_bps} bps ({min_fee_pct:.2}%)\n\
        Keeper fee: {keeper_fee_bps} bps ({keeper_fee_pct:.2}%)\n\
        Min period: {min_period_seconds} seconds\n\
        Default allowance periods: {default_allowance_periods}\n\
        Allowed mint: {allowed_mint}\n\
        Max withdrawal: {max_withdrawal_usdc} USDC\n\
        Max grace period: {max_grace_period_seconds} seconds"
    ))
}
