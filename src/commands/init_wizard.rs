//! Interactive initialization wizard for first-time setup
//!
//! Provides a guided setup experience for new merchants, including:
//! - Pre-flight checks (wallet, SOL balance, RPC connectivity)
//! - Interactive prompts for treasury and fee setup
//! - Merchant initialization
//! - Optional first plan creation

use crate::config::TallyCliConfig;
use anyhow::{anyhow, Context, Result};
use dialoguer::{Confirm, Input};
use std::str::FromStr;
use tally_sdk::solana_sdk::commitment_config::CommitmentConfig;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tally_sdk::{get_usdc_mint, load_keypair, SimpleTallyClient};

/// Minimum SOL balance required for merchant initialization (0.01 SOL for rent + fees)
const MIN_SOL_BALANCE: f64 = 0.01;

/// Lamports per SOL
const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;

/// Convert lamports to SOL
fn lamports_to_sol(lamports: u64) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let lamports_f64 = lamports as f64;
    lamports_f64 / LAMPORTS_PER_SOL
}

/// Execute the interactive initialization wizard
///
/// # Errors
/// Returns error if any step fails (wallet checks, RPC connectivity, merchant initialization)
pub async fn execute(
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    skip_plan: bool,
) -> Result<String> {
    println!("\n🚀 Welcome to Tally! Let's set up your merchant account.\n");

    // Step 1: Pre-flight checks
    println!("Running pre-flight checks...\n");

    // Check wallet
    print!("Checking wallet... ");
    let wallet = load_keypair(None).context("Failed to load wallet")?;
    println!("✓ found at ~/.config/solana/id.json");
    println!("   Address: {}", wallet.pubkey());

    // Check RPC connectivity
    print!("Checking RPC connection... ");
    tally_client
        .get_health()
        .context("Failed to connect to RPC endpoint")?;
    println!("✓ connected");

    // Check wallet balance
    print!("Checking wallet balance... ");
    let balance = tally_client
        .rpc_client
        .get_balance_with_commitment(&wallet.pubkey(), CommitmentConfig::confirmed())
        .context("Failed to get wallet balance")?
        .value;
    let balance_sol = lamports_to_sol(balance);
    println!("✓ {balance_sol:.6} SOL");

    if balance_sol < MIN_SOL_BALANCE {
        return Err(anyhow!(
            "\n❌ Insufficient SOL balance\n\
             \n\
             You have {balance_sol:.6} SOL, but need at least {MIN_SOL_BALANCE:.6} SOL for transaction fees and rent.\n\
             \n\
             Get SOL at:\n\
             • Devnet: https://faucet.solana.com\n\
             • Mainnet: Use a centralized exchange or DEX"
        ));
    }

    println!("\n✅ All pre-flight checks passed!\n");

    // Step 2: Treasury setup
    let treasury_ata = prompt_treasury_setup(tally_client)?;

    // Step 3: Fee setup
    let fee_bps = prompt_fee_setup()?;

    // Step 4: Initialize merchant
    println!("\n⏳ Initializing merchant account...");

    let usdc_mint = get_usdc_mint(None)?;
    let (merchant_pda, signature, created_ata) = tally_client
        .initialize_merchant_with_treasury(&wallet, &usdc_mint, &treasury_ata, fee_bps)
        .context("Failed to initialize merchant")?;

    println!("✅ Merchant account created!\n");

    // Step 5: Display results
    let fee_percentage = config.format_fee_percentage(fee_bps);
    let ata_message = if created_ata {
        "Treasury ATA created"
    } else {
        "Using existing treasury ATA"
    };

    let mut output = format!(
        "Merchant Setup Complete!\n\
         ═══════════════════════════════════════════════════\n\
         \n\
         Merchant PDA:       {}\n\
         Authority:          {}\n\
         Treasury ATA:       {}\n\
         Fee:                {} bps ({:.1}%)\n\
         Transaction:        {}\n\
         Status:             {}\n\
         \n\
         ✓ Merchant PDA saved to config file\n\
         \n\
         Next Steps:\n\
         • Create your first subscription plan\n\
         • Monitor with: tally-merchant dashboard overview\n\
         • View plans with: tally-merchant plan list\n",
        merchant_pda,
        wallet.pubkey(),
        treasury_ata,
        fee_bps,
        fee_percentage,
        signature,
        ata_message
    );

    // Step 6: Optional plan creation guidance
    if !skip_plan {
        println!();
        let create_plan = Confirm::new()
            .with_prompt("Would you like to create your first subscription plan now?")
            .default(true)
            .interact()
            .context("Failed to read user input")?;

        if create_plan {
            use std::fmt::Write;
            output.push_str("\n\nTo create a plan, run:\n\n");
            write!(
                output,
                "  tally-merchant plan create \\\n\
                 --merchant {merchant_pda} \\\n\
                 --id premium \\\n\
                 --name \"Premium Plan\" \\\n\
                 --price-usdc 10.0 \\\n\
                 --period-days 30\n"
            )
            .expect("Writing to String should not fail");
            output.push_str(
                "\nOr use interactive mode:\n\n  tally-merchant plan create --merchant <merchant> --interactive\n",
            );
        }
    }

    output.push_str("\n💡 Run 'tally-merchant --help' to see all available commands.\n");

    Ok(output)
}

/// Prompt for treasury setup
///
/// Asks if user has existing treasury or needs to create one
///
/// # Errors
/// Returns error if user input fails or pubkey parsing fails
fn prompt_treasury_setup(tally_client: &SimpleTallyClient) -> Result<Pubkey> {
    println!("Treasury Setup");
    println!("──────────────────────────────────────────────────");
    println!(
        "A treasury is a USDC token account where subscription\n\
         payments will be deposited.\n"
    );

    let has_treasury = Confirm::new()
        .with_prompt("Do you have an existing USDC treasury account?")
        .default(false)
        .interact()
        .context("Failed to read user input")?;

    if has_treasury {
        let treasury_str: String = Input::new()
            .with_prompt("Enter your treasury address")
            .interact_text()
            .context("Failed to read treasury address")?;

        let treasury = Pubkey::from_str(&treasury_str)
            .context("Invalid treasury address - must be a valid Solana public key")?;

        // Validate that the treasury exists
        tally_client
            .account_exists(&treasury)
            .context("Failed to check treasury account")?
            .then_some(())
            .ok_or_else(|| anyhow!("Treasury account does not exist on-chain"))?;

        println!("✓ Using existing treasury: {treasury}");
        Ok(treasury)
    } else {
        // User needs to create treasury
        println!(
            "\n⚠️  You'll need to create a USDC Associated Token Account (ATA)\n\
             before continuing. The CLI will automatically create it during\n\
             merchant initialization if it doesn't exist.\n"
        );

        let treasury_str: String = Input::new()
            .with_prompt("Enter the treasury address to create/use")
            .interact_text()
            .context("Failed to read treasury address")?;

        let treasury = Pubkey::from_str(&treasury_str)
            .context("Invalid treasury address - must be a valid Solana public key")?;

        println!("✓ Will use treasury: {treasury}");
        Ok(treasury)
    }
}

/// Prompt for fee setup
///
/// Asks user what merchant fee they want to charge
///
/// # Errors
/// Returns error if user input fails or fee percentage is invalid
fn prompt_fee_setup() -> Result<u16> {
    println!("\nMerchant Fee Setup");
    println!("──────────────────────────────────────────────────");
    println!(
        "The merchant fee is YOUR fee charged on top of the\n\
         subscription price. This goes to your treasury.\n\
         \n\
         Recommended: 0.5-2%\n"
    );

    let fee_pct: f64 = Input::new()
        .with_prompt("Fee percentage (0-10%)")
        .default(0.5)
        .interact_text()
        .context("Failed to read fee percentage")?;

    if !(0.0..=10.0).contains(&fee_pct) {
        return Err(anyhow!("Fee must be between 0% and 10%, got {fee_pct:.2}%"));
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let fee_bps = (fee_pct * 100.0) as u16;

    println!("✓ Merchant fee set to {fee_bps} bps ({fee_pct:.1}%)");

    Ok(fee_bps)
}
