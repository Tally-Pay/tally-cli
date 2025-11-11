//! List payment terms command implementation

use crate::utils::formatting::{
    format_payment_terms_human, format_payment_terms_json, PaymentTermsInfo,
};
use anyhow::{anyhow, Context, Result};
use clap::ValueEnum;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::SimpleTallyClient;
use tracing::info;

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Execute the list payment terms command
///
/// # Errors
/// Returns error if payment terms listing fails due to network issues or invalid payee PDA
///
/// # Panics
/// May panic if `period_secs` from on-chain data is below minimum (24 hours).
/// This should not occur under normal operation as values are validated on-chain.
pub async fn execute(
    tally_client: &SimpleTallyClient,
    payee_str: &str,
    output_format: &OutputFormat,
) -> Result<String> {
    info!("Starting payment terms listing for payee: {}", payee_str);

    // Parse payee PDA address
    let payee_pda = Pubkey::from_str(payee_str)
        .map_err(|e| anyhow!("Invalid payee PDA address '{payee_str}': {e}"))?;
    info!("Using payee PDA: {}", payee_pda);

    // Validate payee exists
    if !tally_client
        .account_exists(&payee_pda)
        .context("Failed to check if payee account exists - check RPC connection")?
    {
        return Err(anyhow!(
            "Payee account does not exist at address: {payee_pda}"
        ));
    }

    info!("Querying payment terms using tally-sdk...");

    // Use tally-sdk to get all payment terms for this payee
    let payment_terms_accounts = tally_client
        .list_payment_terms(&payee_pda)
        .context("Failed to fetch payment terms - check RPC connection and payee account state")?;

    info!(
        "Found {} payment terms accounts",
        payment_terms_accounts.len()
    );

    // Parse and format payment terms data
    let mut terms_list = Vec::new();
    for (pubkey, terms) in payment_terms_accounts {
        let terms_info = PaymentTermsInfo {
            address: pubkey,
            terms_id: terms.terms_id_str(),
            amount: tally_sdk::UsdcAmount::from_microlamports(terms.amount_usdc),
            period: tally_sdk::PaymentPeriod::from_seconds(terms.period_secs)
                .unwrap_or_else(|_| tally_sdk::PaymentPeriod::days(1).expect("Default period")),
        };
        info!("Parsed payment terms: {}", terms_info.terms_id);
        terms_list.push(terms_info);
    }

    // Sort payment terms by terms_id for consistent output
    terms_list.sort_by(|a, b| a.terms_id.cmp(&b.terms_id));

    // Format output based on requested format
    match output_format {
        OutputFormat::Human => Ok(format_payment_terms_human(&terms_list, &payee_pda)),
        OutputFormat::Json => format_payment_terms_json(&terms_list),
    }
}
