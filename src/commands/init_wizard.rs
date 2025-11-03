//! Interactive initialization wizard for first-time setup
//!
//! Provides a guided setup experience for new merchants, including:
//! - Interactive wallet selection with balance display
//! - Pre-flight checks (RPC connectivity, SOL balance)
//! - Interactive prompts for treasury and fee setup
//! - Merchant initialization
//! - Optional first plan creation

use crate::config::TallyCliConfig;
use anyhow::{anyhow, Context, Result};
use dialoguer::{Confirm, Input, Select};
use std::str::FromStr;
use tally_sdk::solana_sdk::commitment_config::CommitmentConfig;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::{Keypair, Signer};
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
/// Returns error if any step fails (wallet selection, RPC connectivity, merchant initialization)
pub async fn execute(
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    skip_plan: bool,
) -> Result<String> {
    println!("\n🚀 Welcome to Tally! Let's set up your merchant account.\n");

    // Step 1: Wallet selection (with info display and progressive disclosure)
    let wallet = prompt_wallet_selection(tally_client)?;

    // Step 2: Pre-flight checks with selected wallet
    println!("\nRunning pre-flight checks...\n");

    // Check RPC connectivity
    print!("Checking RPC connection... ");
    tally_client
        .get_health()
        .context("Failed to connect to RPC endpoint")?;
    println!("✓ connected");

    // Check wallet balance with recovery flow
    print!("Checking wallet balance... ");
    let balance = tally_client
        .rpc_client
        .get_balance_with_commitment(&wallet.pubkey(), CommitmentConfig::confirmed())
        .context("Failed to get wallet balance")?
        .value;
    let balance_sol = lamports_to_sol(balance);
    println!("✓ {balance_sol:.6} SOL");

    if balance_sol < MIN_SOL_BALANCE {
        handle_insufficient_balance(balance_sol)?;
    }

    println!("\n✅ All pre-flight checks passed!\n");

    // Step 3: Treasury setup
    let treasury_ata = prompt_treasury_setup(tally_client)?;

    // Step 4: Fee setup
    let fee_bps = prompt_fee_setup()?;

    // Step 5: Initialize merchant
    println!("\n⏳ Initializing merchant account...");

    let usdc_mint = get_usdc_mint(None)?;
    let (merchant_pda, signature, created_ata) = tally_client
        .initialize_merchant_with_treasury(&wallet, &usdc_mint, &treasury_ata, fee_bps)
        .context("Failed to initialize merchant")?;

    println!("✅ Merchant account created!\n");

    // Step 6: Display results
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

    // Step 7: Optional plan creation guidance
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

/// Prompt for wallet selection with info display and progressive disclosure
///
/// Shows default wallet info (address, balance) and asks for confirmation.
/// If user declines, prompts for custom wallet path.
///
/// # Errors
/// Returns error if wallet cannot be loaded or RPC calls fail
fn prompt_wallet_selection(tally_client: &SimpleTallyClient) -> Result<Keypair> {
    println!("Wallet Setup");
    println!("──────────────────────────────────────────────────");
    println!(
        "The merchant authority wallet will be used to:\n\
         • Create and manage subscription plans\n\
         • Update merchant settings\n\
         • Withdraw merchant fees\n"
    );

    // Try to load default wallet
    let default_path = "~/.config/solana/id.json";

    if let Ok(wallet) = load_keypair(None) {
        // Get balance for display
        let balance = tally_client
            .rpc_client
            .get_balance_with_commitment(&wallet.pubkey(), CommitmentConfig::confirmed())
            .context("Failed to get wallet balance")?
            .value;
        let balance_sol = lamports_to_sol(balance);

        // Show wallet info
        println!("Found wallet: {default_path}");
        println!("Address: {}", wallet.pubkey());
        println!("Balance: {balance_sol:.6} SOL\n");

        // Ask confirmation
        let use_default = Confirm::new()
            .with_prompt("Use this wallet as merchant authority?")
            .default(true)
            .interact()
            .context("Failed to read user input")?;

        if use_default {
            Ok(wallet)
        } else {
            // User declined, ask for custom path
            prompt_custom_wallet_path()
        }
    } else {
        // No default wallet found
        println!("⚠️  No default Solana wallet found at {default_path}\n");
        prompt_custom_wallet_path()
    }
}

/// Prompt for custom wallet path with validation loop
///
/// # Errors
/// Returns error if user input fails (this is terminal - exits the program)
fn prompt_custom_wallet_path() -> Result<Keypair> {
    loop {
        let path: String = Input::new()
            .with_prompt("Enter wallet path")
            .interact_text()
            .context("Failed to read user input")?;

        // Expand tilde to home directory if needed
        let expanded_path = expand_tilde(&path);

        match load_keypair(Some(&expanded_path)) {
            Ok(wallet) => {
                println!("✓ Wallet loaded successfully");
                println!("   Address: {}\n", wallet.pubkey());
                return Ok(wallet);
            }
            Err(e) => {
                println!("❌ Failed to load wallet: {e}\n");
                // Loop continues, prompting again
            }
        }
    }
}

/// Expand tilde (~) to home directory path
///
/// Converts paths like `~/foo` to `/home/user/foo`
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replacen('~', &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

/// Handle insufficient balance with recovery options
///
/// Displays error message and offers actionable choices to the user.
///
/// # Errors
/// Returns error after user makes a choice (to exit the wizard)
fn handle_insufficient_balance(balance_sol: f64) -> Result<()> {
    println!(
        "\n❌ Insufficient SOL balance\n\
         \n\
         You have {balance_sol:.6} SOL, but need at least {MIN_SOL_BALANCE:.6} SOL\n\
         for transaction fees and rent.\n\
         \n\
         Get SOL at:\n\
         • Devnet: https://faucet.solana.com\n\
         • Mainnet: Use a centralized exchange or DEX\n"
    );

    let choices = vec![
        "Fund this wallet and retry",
        "Use a different wallet",
        "Exit setup",
    ];

    let selection = Select::new()
        .with_prompt("What would you like to do?")
        .items(&choices)
        .default(0)
        .interact()
        .context("Failed to read user input")?;

    match selection {
        0 => {
            // Fund and retry - currently just exits with helpful message
            Err(anyhow!(
                "Please fund your wallet with at least {MIN_SOL_BALANCE:.6} SOL,\n\
                 then run 'tally-merchant init' again."
            ))
        }
        1 => {
            // Use different wallet - currently just exits with helpful message
            Err(anyhow!(
                "Please run 'tally-merchant init' again and select a different wallet\n\
                 when prompted during the wallet setup step."
            ))
        }
        2 => {
            // Exit
            Err(anyhow!("Setup canceled by user"))
        }
        _ => unreachable!(),
    }
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
