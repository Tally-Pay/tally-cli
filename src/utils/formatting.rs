//! Output formatting utilities for the Tally CLI

use crate::config::TallyCliConfig;
use anyhow::Result;
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::{PaymentPeriod, UsdcAmount};

/// Detect network type from RPC URL
///
/// Returns a human-readable network name based on the RPC URL pattern:
/// - "localnet" for localhost/127.0.0.1
/// - "devnet" for devnet.solana.com
/// - "testnet" for testnet.solana.com
/// - "mainnet" for mainnet-beta.solana.com
/// - "custom" for other URLs
#[must_use]
pub fn detect_network(rpc_url: &str) -> String {
    let url_lower = rpc_url.to_lowercase();

    if url_lower.contains("localhost") || url_lower.contains("127.0.0.1") {
        "localnet".to_string()
    } else if url_lower.contains("devnet") {
        "devnet".to_string()
    } else if url_lower.contains("testnet") {
        "testnet".to_string()
    } else if url_lower.contains("mainnet") {
        "mainnet".to_string()
    } else {
        "custom".to_string()
    }
}

/// Format unix timestamp to human-readable date
///
/// Returns "Invalid" for timestamps that cannot be converted to valid dates,
/// or "N/A" for zero/negative timestamps.
#[must_use]
pub fn format_timestamp(timestamp: i64) -> String {
    if timestamp <= 0 {
        return "N/A".to_string();
    }

    // Safely convert i64 to u64, handling negative timestamps
    let timestamp_u64 = u64::try_from(timestamp).unwrap_or(0);

    SystemTime::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(timestamp_u64))
        .and_then(|datetime| {
            // Safely calculate duration since epoch without panicking
            datetime.duration_since(UNIX_EPOCH).ok()
        })
        .map_or_else(
            || format!("Invalid timestamp: {timestamp}"),
            |duration_since_epoch| {
                let secs = duration_since_epoch.as_secs();
                let days = secs / 86400;
                let hours = (secs % 86400) / 3600;
                let minutes = (secs % 3600) / 60;
                let seconds = secs % 60;

                // Calculate approximate date (this is a simplified calculation)
                let years_since_1970 = days / 365;
                let remaining_days = days % 365;
                let months = remaining_days / 30;
                let day_of_month = remaining_days % 30;

                format!(
                    "{}-{:02}-{:02} {:02}:{:02}:{:02}",
                    1970 + years_since_1970,
                    months + 1,
                    day_of_month + 1,
                    hours,
                    minutes,
                    seconds
                )
            },
        )
}

// Payment Terms formatting structures and functions

#[derive(Debug, Clone)]
pub struct PaymentTermsInfo {
    pub address: Pubkey,
    pub terms_id: String,
    pub amount: UsdcAmount,
    pub period: PaymentPeriod,
}

#[derive(Debug, Clone)]
pub struct AgreementInfo {
    pub address: Pubkey,
    pub payment_terms: Pubkey,
    pub payer: Pubkey,
    pub next_payment_ts: i64,
    pub active: bool,
    pub payment_count: u32,
    pub created_ts: i64,
    pub last_amount: UsdcAmount,
}

/// Format payment terms for human-readable output
#[must_use]
pub fn format_payment_terms_human(terms_list: &[PaymentTermsInfo], payee_pda: &Pubkey) -> String {
    use crate::utils::colors::Theme;
    use std::fmt::Write;

    let mut output = String::new();
    writeln!(&mut output, "{}", Theme::header("Payment Terms for Payee")).unwrap();
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Payee:"),
        Theme::highlight(&payee_pda.to_string())
    )
    .unwrap();
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Total Terms:"),
        Theme::value(&terms_list.len().to_string())
    )
    .unwrap();
    writeln!(&mut output).unwrap();

    if terms_list.is_empty() {
        writeln!(
            &mut output,
            "{}",
            Theme::warning("No payment terms found for this payee")
        )
        .unwrap();
    } else {
        for terms in terms_list {
            writeln!(
                &mut output,
                "{} {}",
                Theme::info("Terms ID:"),
                Theme::value(&terms.terms_id)
            )
            .unwrap();
            writeln!(
                &mut output,
                "  {} {}",
                Theme::dim("Amount:"),
                terms.amount
            )
            .unwrap();
            writeln!(&mut output, "  {} {}", Theme::dim("Period:"), terms.period).unwrap();
            writeln!(
                &mut output,
                "  {} {}",
                Theme::dim("Address:"),
                Theme::dim(&terms.address.to_string())
            )
            .unwrap();
            writeln!(&mut output).unwrap();
        }
    }

    output
}

/// Format payment terms for JSON output
///
/// # Errors
///
/// Returns an error if JSON serialization fails
pub fn format_payment_terms_json(terms_list: &[PaymentTermsInfo]) -> Result<String> {
    let json = serde_json::to_string_pretty(&serde_json::json!({
        "payment_terms": terms_list.iter().map(|t| serde_json::json!({
            "address": t.address.to_string(),
            "terms_id": t.terms_id,
            "amount_usdc": t.amount.usdc(),
            "amount_microlamports": t.amount.microlamports(),
            "period_seconds": t.period.seconds(),
            "period_days": t.period.as_days(),
            "period_display": t.period.to_string(),
        })).collect::<Vec<_>>(),
        "count": terms_list.len(),
    }))?;
    Ok(json)
}

/// Format payment agreements for human-readable output
#[must_use]
pub fn format_agreements_human(
    agreements: &[AgreementInfo],
    payment_terms_pda: &Pubkey,
    _config: &TallyCliConfig,
) -> String {
    use crate::utils::colors::Theme;
    use std::fmt::Write;

    let mut output = String::new();
    writeln!(&mut output, "{}", Theme::header("Payment Agreements")).unwrap();
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Payment Terms:"),
        Theme::highlight(&payment_terms_pda.to_string())
    )
    .unwrap();
    writeln!(
        &mut output,
        "{} {}",
        Theme::info("Total Agreements:"),
        Theme::value(&agreements.len().to_string())
    )
    .unwrap();
    writeln!(&mut output).unwrap();

    if agreements.is_empty() {
        writeln!(
            &mut output,
            "{}",
            Theme::warning("No payment agreements found for these terms")
        )
        .unwrap();
    } else {
        for agreement in agreements {
            let status = if agreement.active {
                Theme::active("Active")
            } else {
                Theme::inactive("Paused")
            };
            writeln!(
                &mut output,
                "{} {}",
                Theme::info("Payer:"),
                Theme::value(&agreement.payer.to_string())
            )
            .unwrap();
            writeln!(&mut output, "  {} {}", Theme::dim("Status:"), status).unwrap();
            writeln!(
                &mut output,
                "  {} {}",
                Theme::dim("Payment Count:"),
                agreement.payment_count
            )
            .unwrap();
            writeln!(
                &mut output,
                "  {} {}",
                Theme::dim("Last Amount:"),
                agreement.last_amount
            )
            .unwrap();
            writeln!(
                &mut output,
                "  {} {}",
                Theme::dim("Next Payment:"),
                format_timestamp(agreement.next_payment_ts)
            )
            .unwrap();
            writeln!(
                &mut output,
                "  {} {}",
                Theme::dim("Address:"),
                Theme::dim(&agreement.address.to_string())
            )
            .unwrap();
            writeln!(&mut output).unwrap();
        }
    }

    output
}

/// Format payment agreements for JSON output
///
/// # Errors
///
/// Returns an error if JSON serialization fails
pub fn format_agreements_json(
    agreements: &[AgreementInfo],
    _config: &TallyCliConfig,
) -> Result<String> {
    let json = serde_json::to_string_pretty(&serde_json::json!({
        "agreements": agreements.iter().map(|a| serde_json::json!({
            "address": a.address.to_string(),
            "payment_terms": a.payment_terms.to_string(),
            "payer": a.payer.to_string(),
            "next_payment_ts": a.next_payment_ts,
            "next_payment_human": format_timestamp(a.next_payment_ts),
            "active": a.active,
            "payment_count": a.payment_count,
            "last_amount_microlamports": a.last_amount.microlamports(),
            "last_amount_usdc": a.last_amount.usdc(),
            "last_amount_display": a.last_amount.to_string(),
            "created_ts": a.created_ts,
            "created_human": format_timestamp(a.created_ts),
        })).collect::<Vec<_>>(),
        "count": agreements.len(),
    }))?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_network_localnet() {
        assert_eq!(detect_network("http://localhost:8899"), "localnet");
        assert_eq!(detect_network("http://127.0.0.1:8899"), "localnet");
        assert_eq!(detect_network("https://localhost:8899"), "localnet");
    }

    #[test]
    fn test_detect_network_devnet() {
        assert_eq!(detect_network("https://api.devnet.solana.com"), "devnet");
        assert_eq!(detect_network("http://api.devnet.solana.com"), "devnet");
    }

    #[test]
    fn test_detect_network_testnet() {
        assert_eq!(detect_network("https://api.testnet.solana.com"), "testnet");
    }

    #[test]
    fn test_detect_network_mainnet() {
        assert_eq!(
            detect_network("https://api.mainnet-beta.solana.com"),
            "mainnet"
        );
        assert_eq!(detect_network("https://mainnet.helius-rpc.com"), "mainnet");
    }

    #[test]
    fn test_detect_network_custom() {
        assert_eq!(detect_network("https://custom-rpc.example.com"), "custom");
        assert_eq!(detect_network("https://my-private-node.com:8899"), "custom");
    }

    #[test]
    fn test_detect_network_case_insensitive() {
        assert_eq!(detect_network("HTTPS://API.DEVNET.SOLANA.COM"), "devnet");
        assert_eq!(detect_network("HTTP://LOCALHOST:8899"), "localnet");
    }
}
