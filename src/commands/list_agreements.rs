//! List payment agreements command implementation

use crate::{
    config::TallyCliConfig,
    utils::formatting::{format_agreements_human, format_agreements_json, AgreementInfo},
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

/// Execute the list agreements command
///
/// # Errors
/// Returns error if agreement listing fails due to network issues or invalid payment terms PDA
pub async fn execute(
    tally_client: &SimpleTallyClient,
    payment_terms_str: &str,
    output_format: &OutputFormat,
    config: &TallyCliConfig,
) -> Result<String> {
    info!(
        "Starting payment agreement listing for payment terms: {}",
        payment_terms_str
    );

    // Parse payment terms PDA address
    let payment_terms_pda = Pubkey::from_str(payment_terms_str)
        .map_err(|e| anyhow!("Invalid payment terms PDA address '{payment_terms_str}': {e}"))?;
    info!("Using payment terms PDA: {}", payment_terms_pda);

    // Validate payment terms exists
    if !tally_client
        .account_exists(&payment_terms_pda)
        .context("Failed to check if payment terms account exists - check RPC connection")?
    {
        return Err(anyhow!(
            "Payment terms account does not exist at address: {payment_terms_pda}"
        ));
    }

    info!("Querying payment agreements using tally-sdk...");

    // Use tally-sdk to get all agreements for this payment terms
    let agreement_accounts = tally_client
        .list_payment_agreements(&payment_terms_pda)
        .context(
            "Failed to fetch agreements - check RPC connection and payment terms account state",
        )?;

    info!(
        "Found {} payment agreement accounts",
        agreement_accounts.len()
    );

    // Parse and format agreement data
    let mut agreements = Vec::new();
    for (pubkey, agreement) in agreement_accounts {
        let agreement_info = AgreementInfo {
            address: pubkey,
            payment_terms: agreement.payment_terms,
            payer: agreement.payer,
            next_payment_ts: agreement.next_payment_ts,
            active: agreement.active,
            payment_count: agreement.payment_count,
            created_ts: agreement.created_ts,
            last_amount: tally_sdk::UsdcAmount::from_microlamports(agreement.last_amount),
        };
        info!(
            "Parsed payment agreement: {} (payer: {})",
            pubkey, agreement.payer
        );
        agreements.push(agreement_info);
    }

    // Sort agreements by created timestamp for consistent output
    agreements.sort_by(|a, b| a.created_ts.cmp(&b.created_ts));

    // Format output based on requested format
    match output_format {
        OutputFormat::Human => Ok(format_agreements_human(
            &agreements,
            &payment_terms_pda,
            config,
        )),
        OutputFormat::Json => format_agreements_json(&agreements, config),
    }
}
