//! Interactive initialization wizard for first-time setup
//!
//! Provides a guided setup experience for new merchants, including:
//! - Interactive wallet selection with balance display
//! - Pre-flight checks (RPC connectivity, SOL balance)
//! - Interactive prompts for treasury and fee setup
//! - Merchant initialization
//! - Optional first plan creation

use crate::config::TallyCliConfig;
use crate::errors::enhance_merchant_init_error;
use crate::utils::formatting::detect_network;
use crate::utils::progress;
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
    _config: &TallyCliConfig,
    skip_plan: bool,
) -> Result<String> {
    println!("\n🚀 Welcome to Tally! Let's set up your merchant account.\n");

    // Step 1: Wallet selection (with info display and progressive disclosure)
    let wallet = prompt_wallet_selection(tally_client)?;

    // Step 2: Pre-flight checks with selected wallet
    println!("\nRunning pre-flight checks...\n");

    // Check RPC connectivity
    let rpc_url = tally_client.rpc_client.url();
    let network = detect_network(&rpc_url);
    print!("Checking RPC connection... ");
    tally_client
        .get_health()
        .context("Failed to connect to RPC endpoint")?;
    println!("✓ connected to {network}");

    // Check wallet balance with recovery flow
    println!("Checking wallet balance for {}...", wallet.pubkey());
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
    let treasury_ata = prompt_treasury_setup(tally_client, &wallet)?;

    // Step 4: Initialize merchant
    println!("\nInitializing merchant account...");
    println!("   • Your merchant will be created on the Free tier (2.0% platform fee)");
    println!("   • Contact platform authority to upgrade to Pro (1.5%) or Enterprise (1.0%)\n");

    let usdc_mint = get_usdc_mint(None)?;

    // Use progress spinner for transaction
    let spinner = progress::create_spinner("Submitting merchant initialization transaction...");
    let result = tally_client
        .initialize_merchant_with_treasury(&wallet, &usdc_mint, &treasury_ata)
        .map_err(|e| enhance_merchant_init_error(&e, &wallet.pubkey(), &treasury_ata));

    match &result {
        Ok(_) => progress::finish_progress_success(&spinner, "Merchant account created"),
        Err(_) => progress::finish_progress_error(&spinner, "Failed to create merchant account"),
    }

    let (merchant_pda, signature, created_ata) = result?;
    println!();

    // Step 5: Display results
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
         Tier:               Free (platform fee: 200 bps / 2.0%)\n\
         Transaction:        {}\n\
         Status:             {}\n\
         \n\
         ✓ Merchant PDA saved to config file\n\
         \n\
         Note: New merchants start on the Free tier.\n\
         Contact the platform authority to upgrade tiers.\n\
         \n\
         Next Steps:\n\
         • Create your first subscription plan\n\
         • Monitor with: tally-merchant dashboard overview\n\
         • View plans with: tally-merchant plan list\n",
        merchant_pda,
        wallet.pubkey(),
        treasury_ata,
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
/// Asks if user has existing treasury or needs to create one.
/// If user doesn't have existing treasury, calculates and displays the default ATA address
/// that will be created, allowing them to press Enter to use it or override with custom address.
///
/// # Errors
/// Returns error if user input fails or pubkey parsing fails
fn prompt_treasury_setup(tally_client: &SimpleTallyClient, wallet: &Keypair) -> Result<Pubkey> {
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
        // Calculate the default ATA address upfront
        let usdc_mint = get_usdc_mint(None)?;
        let default_ata = tally_sdk::ata::get_associated_token_address_for_mint(
            &wallet.pubkey(),
            &usdc_mint,
        )?;

        // Display the default ATA that will be created
        println!(
            "\n💡 The CLI will automatically create a USDC treasury (ATA) for you.\n"
        );
        println!("Default treasury address: {default_ata}");
        println!("                          (will be created if it doesn't exist)\n");

        // Prompt with default, allowing Enter to accept or custom address to override
        let treasury_str: String = Input::new()
            .with_prompt("Treasury address (press Enter for default, or enter custom address)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to read treasury address")?;

        let treasury = if treasury_str.trim().is_empty() {
            // User pressed Enter - use the calculated default
            println!("✓ Using default treasury: {default_ata}");
            default_ata
        } else {
            // User provided custom address - parse and use it
            let custom_treasury = Pubkey::from_str(treasury_str.trim())
                .context("Invalid treasury address - must be a valid Solana public key")?;
            println!("✓ Using custom treasury: {custom_treasury}");
            custom_treasury
        };

        Ok(treasury)
    }
}
