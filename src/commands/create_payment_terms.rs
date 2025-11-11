//! Create payment terms command implementation

use crate::config::TallyCliConfig;
use crate::utils::colors::Theme;
use crate::utils::progress;
use anyhow::{anyhow, Context, Result};
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::solana_sdk::signature::Signer;
use tally_sdk::{
    load_keypair, pda_v2, program_types::CreatePaymentTermsArgs, PaymentPeriod, SimpleTallyClient,
    TermsId, UsdcAmount,
};
use tracing::info;

/// Arguments for creating payment terms
pub struct CreatePaymentTermsRequest<'a> {
    pub payee_str: &'a str,
    pub terms_id: &'a str,
    pub amount_usdc_float: f64,
    pub period_days: u64,
    pub authority_path: Option<&'a str>,
}

/// Execute the create payment terms command
///
/// # Errors
/// Returns error if payment terms creation fails due to invalid parameters, network issues, or Solana program errors
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &CreatePaymentTermsRequest<'_>,
    _config: &TallyCliConfig,
) -> Result<String> {
    info!("Starting payment terms creation");

    // Parse payee PDA address (for validation)
    let expected_payee_pda = Pubkey::from_str(request.payee_str)
        .map_err(|e| anyhow!("Invalid payee PDA address '{}': {}", request.payee_str, e))?;
    info!("Expected payee PDA: {expected_payee_pda}");

    // Load authority keypair
    let authority =
        load_keypair(request.authority_path).context("Failed to load authority keypair")?;
    info!("Using authority: {}", authority.pubkey());

    // Validate authority matches the provided payee PDA
    let authority_pubkey = Pubkey::from(authority.pubkey().to_bytes());
    let computed_payee_pda: Pubkey = pda_v2::payee(&authority_pubkey)?.into();
    if expected_payee_pda != computed_payee_pda {
        return Err(anyhow!(
            "Authority mismatch: expected payee PDA {} for authority {}, but got {}",
            computed_payee_pda,
            authority.pubkey(),
            expected_payee_pda
        ));
    }

    // Create type-safe domain types
    let terms_id = TermsId::new(request.terms_id)
        .context("Invalid terms ID - use only alphanumeric, underscores, and hyphens")?;
    let amount = UsdcAmount::from_usdc(request.amount_usdc_float);
    let period = PaymentPeriod::days(request.period_days)
        .context("Invalid period - must be at least 1 day")?;

    info!(
        "Creating payment terms: id={}, amount={}, period={}",
        terms_id, amount, period
    );

    // Convert to SDK types (until SDK is fully migrated)
    let terms_id_bytes = terms_id.to_padded_bytes();
    let period_secs = period.seconds();

    let terms_args = CreatePaymentTermsArgs {
        terms_id: terms_id.as_str().to_string(),
        terms_id_bytes,
        amount_usdc: amount.microlamports(),
        period_secs,
    };

    // Use tally-sdk's high-level convenience method with progress indicator
    let spinner = progress::create_spinner("Creating payment terms and submitting transaction...");
    let result = tally_client
        .create_payment_terms(&authority, terms_args)
        .map_err(|e| anyhow!("Failed to create payment terms: {e}"));

    match &result {
        Ok((_, signature)) => {
            progress::finish_progress_success(&spinner, "Payment terms created successfully");
            info!("Transaction confirmed: {}", signature);
        }
        Err(_) => {
            progress::finish_progress_error(&spinner, "Failed to create payment terms");
        }
    }

    let (payment_terms_pda, signature) = result?;

    // Build output with colors - use write! to construct string
    let mut output = String::new();
    writeln!(
        &mut output,
        "{}",
        Theme::success("Payment terms created successfully!")
    )?;
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Payment Terms PDA:"),
        Theme::highlight(&payment_terms_pda.to_string())
    )?;
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Terms ID:"),
        Theme::value(terms_id.as_str())
    )?;
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Amount:"),
        Theme::value(&amount.to_string())
    )?;
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Period:"),
        Theme::value(&period.to_string())
    )?;

    // Write signature - using format! directly to avoid clippy warnings
    write!(
        &mut output,
        "{} {signature}",
        Theme::info("Transaction signature:")
    )?;

    Ok(output)
}
