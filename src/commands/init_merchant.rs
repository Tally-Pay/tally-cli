//! Init merchant command implementation

use crate::config::TallyCliConfig;
use crate::config_file::ConfigFile;
use crate::errors::enhance_merchant_init_error;
use anyhow::{anyhow, Result};
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tally_sdk::{get_usdc_mint, load_keypair, SimpleTallyClient};
use tracing::info;

/// Execute the init merchant command
///
/// # Errors
/// Returns error if merchant initialization fails due to invalid parameters, network issues, or Solana program errors
#[allow(clippy::cognitive_complexity)] // Complex validation logic for merchant initialization
pub async fn execute(
    tally_client: &SimpleTallyClient,
    authority_path: Option<&str>,
    treasury_str: &str,
    usdc_mint_str: Option<&str>,
    _config: &TallyCliConfig,
) -> Result<String> {
    info!("Starting merchant initialization");

    // Load authority keypair
    let authority = load_keypair(authority_path)?;
    info!("Using authority: {}", authority.pubkey());

    // Parse USDC mint
    let usdc_mint =
        get_usdc_mint(usdc_mint_str).map_err(|e| anyhow!("Failed to parse USDC mint: {e}"))?;
    info!("Using USDC mint: {}", usdc_mint);

    // Parse treasury ATA
    let treasury_ata = Pubkey::from_str(treasury_str)
        .map_err(|e| anyhow!("Invalid treasury ATA address '{treasury_str}': {e}"))?;
    info!("Using treasury ATA: {}", treasury_ata);

    // Use the new unified method that handles both ATA existence scenarios
    // Platform fee is automatically set to Free tier (2.0%) by the program
    let (merchant_pda, signature, created_ata) = tally_client
        .initialize_merchant_with_treasury(&authority, &usdc_mint, &treasury_ata)
        .map_err(|e| enhance_merchant_init_error(&e, &authority.pubkey(), &treasury_ata))?;

    info!(
        "Transaction confirmed: {}, created_ata: {}",
        signature, created_ata
    );

    // Save merchant PDA to config file for future use
    let config_save_result = save_merchant_to_config(&merchant_pda);
    let config_message = match config_save_result {
        Ok(profile_name) => format!("\n✓ Merchant PDA saved to config (profile: {profile_name})"),
        Err(e) => format!("\n⚠ Could not save merchant PDA to config: {e}"),
    };

    // Return success message with merchant PDA, transaction signature, and ATA creation info
    let ata_message = if created_ata {
        "Treasury ATA created and merchant initialized"
    } else {
        "Merchant initialized with existing treasury ATA"
    };

    Ok(format!(
        "Merchant initialization successful!\n{}\nMerchant PDA: {}\nTransaction signature: {}\nAuthority: {}\nTreasury ATA: {}\nTier: Free (platform fee: 200 bps / 2.0%)\n\n\
        Note: New merchants start on the Free tier.\n\
        Contact the platform authority to upgrade to Pro (1.5%) or Enterprise (1.0%) tiers.{}",
        ata_message,
        merchant_pda,
        signature,
        authority.pubkey(),
        treasury_ata,
        config_message
    ))
}

/// Save merchant PDA to config file
///
/// Returns the profile name where the merchant was saved
fn save_merchant_to_config(merchant_pda: &Pubkey) -> Result<String> {
    let mut config_file = ConfigFile::load().unwrap_or_else(|_| ConfigFile::new());

    // Get active profile name before setting merchant
    let profile_name = config_file
        .defaults
        .active_profile
        .clone()
        .unwrap_or_else(|| "devnet".to_string());

    config_file.set_merchant(merchant_pda.to_string())?;
    config_file.save()?;

    Ok(profile_name)
}
