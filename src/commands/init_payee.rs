//! Init payee command implementation

use crate::config::TallyCliConfig;
use crate::config_file::ConfigFile;
use crate::errors::enhance_payee_init_error;
use crate::utils::colors::Theme;
use anyhow::{anyhow, Result};
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tally_sdk::{get_usdc_mint, load_keypair, SimpleTallyClient};
use tracing::info;

/// Execute the init payee command
///
/// # Errors
/// Returns error if payee initialization fails due to invalid parameters, network issues, or Solana program errors
pub async fn execute(
    tally_client: &SimpleTallyClient,
    authority_path: Option<&str>,
    treasury_str: &str,
    usdc_mint_str: Option<&str>,
    _config: &TallyCliConfig,
) -> Result<String> {
    info!("Starting payee initialization");

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
    // Volume tier is automatically set to Standard by the program
    let (payee_pda, signature, created_ata) = tally_client
        .init_payee_with_treasury(&authority, &treasury_ata, &usdc_mint)
        .map_err(|e| enhance_payee_init_error(&e, &authority.pubkey(), &treasury_ata))?;

    info!(
        "Transaction confirmed: {}, created_ata: {}",
        signature.as_str(), created_ata
    );

    // Save payee PDA to config file for future use
    let config_save_result = save_payee_to_config(&payee_pda);

    // Build colored output
    let mut output = String::new();
    writeln!(
        &mut output,
        "{}",
        Theme::success("Payee initialization successful!")
    )?;

    // Show ATA creation status
    if created_ata {
        writeln!(
            &mut output,
            "{}",
            Theme::info("Treasury ATA created and payee initialized")
        )?;
    } else {
        writeln!(
            &mut output,
            "{}",
            Theme::info("Payee initialized with existing treasury ATA")
        )?;
    }

    writeln!(&mut output)?;
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Payee PDA:"),
        Theme::highlight(&payee_pda.to_string())
    )?;
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Transaction signature:"),
        Theme::dim(&signature)
    )?;
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Authority:"),
        Theme::value(&authority.pubkey().to_string())
    )?;
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Treasury ATA:"),
        Theme::value(&treasury_ata.to_string())
    )?;
    writeln!(
        &mut output,
        "{} {} (platform fee: 25 bps / 0.25%)",
        Theme::info("Volume Tier:"),
        Theme::active("Standard")
    )?;

    // Config save message
    match config_save_result {
        Ok(profile_name) => {
            writeln!(&mut output)?;
            writeln!(
                &mut output,
                "{} Payee PDA saved to config (profile: {})",
                Theme::success("✓"),
                Theme::value(&profile_name)
            )?;
        }
        Err(e) => {
            writeln!(&mut output)?;
            writeln!(
                &mut output,
                "{} Could not save payee PDA to config: {}",
                Theme::warning("⚠"),
                Theme::dim(&e.to_string())
            )?;
        }
    }

    // Note about volume tiers
    writeln!(&mut output)?;
    writeln!(
        &mut output,
        "{}",
        Theme::dim("Note: All payees start at Standard tier (0.25% platform fee).")
    )?;
    write!(&mut output, "{}", Theme::dim("Volume tiers auto-upgrade based on monthly payment volume (Growth: 0.20%, Scale: 0.15%)."))?;

    Ok(output)
}

/// Save payee PDA to config file
///
/// Returns the profile name where the payee was saved
fn save_payee_to_config(payee_pda: &Pubkey) -> Result<String> {
    let mut config_file = ConfigFile::load().unwrap_or_else(|_| ConfigFile::new());

    // Get active profile name before setting payee
    let profile_name = config_file
        .defaults
        .active_profile
        .clone()
        .unwrap_or_else(|| "devnet".to_string());

    config_file.set_payee(payee_pda.to_string())?;
    config_file.save()?;

    Ok(profile_name)
}
