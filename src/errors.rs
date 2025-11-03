//! Enhanced error handling with actionable recovery suggestions
//!
//! This module provides error types and functions that give users
//! clear guidance on how to fix common problems.

use anyhow::{anyhow, Result};
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;

/// Parse a merchant PDA with enhanced error messages
///
/// Provides helpful suggestions if parsing fails, including:
/// - Checking saved merchant in config
/// - Running init wizard
/// - Examples of valid addresses
///
/// # Errors
/// Returns enhanced error with recovery suggestions if parsing fails
pub fn parse_merchant_pda(
    merchant_str: &str,
    config_merchant: Option<&Pubkey>,
) -> Result<Pubkey> {
    Pubkey::from_str(merchant_str).map_err(|e| {
        let mut error_msg = format!(
            "Invalid merchant address: '{merchant_str}'\n\n\
             Merchant addresses must be base58-encoded Solana public keys (44 characters).\n\n\
             Did you mean to:"
        );

        // Suggest using saved merchant if available
        if let Some(saved_merchant) = config_merchant {
            write!(
                error_msg,
                "\n  • Use your saved merchant: {saved_merchant}"
            )
            .expect("Writing to String should not fail");
            error_msg.push_str("\n  • Or update it with: tally-merchant config set merchant <NEW_ADDRESS>");
        } else {
            error_msg.push_str("\n  • Run 'tally-merchant init' to create a new merchant?");
            error_msg.push_str("\n  • Check your merchant address with: tally-merchant config get merchant");
        }

        error_msg.push_str("\n\nExample valid address: HkDq7K2RRStvPrXw6U3YPJrPU2dYbvGj8Y5z8VQmKR8C");
        write!(error_msg, "\n\nOriginal error: {e}").expect("Writing to String should not fail");

        anyhow!(error_msg)
    })
}

/// Parse a plan PDA with enhanced error messages
///
/// # Errors
/// Returns enhanced error with recovery suggestions if parsing fails
pub fn parse_plan_pda(plan_str: &str, merchant: Option<&Pubkey>) -> Result<Pubkey> {
    Pubkey::from_str(plan_str).map_err(|e| {
        let mut error_msg = format!(
            "Invalid plan address: '{plan_str}'\n\n\
             Plan addresses must be base58-encoded Solana public keys (44 characters).\n\n\
             Did you mean to:"
        );

        if let Some(merchant_pda) = merchant {
            write!(
                error_msg,
                "\n  • List your plans with: tally-merchant plan list --merchant {merchant_pda}"
            )
            .expect("Writing to String should not fail");
        } else {
            error_msg.push_str("\n  • List your plans with: tally-merchant plan list --merchant <MERCHANT_PDA>");
        }

        error_msg.push_str("\n  • Create a new plan with: tally-merchant plan create --help");

        error_msg.push_str("\n\nExample valid address: 8rPqJKt2fT9xYw5zR3vN8mPdLkQcXnU1wVbHjGaFsYe4");
        write!(error_msg, "\n\nOriginal error: {e}").expect("Writing to String should not fail");

        anyhow!(error_msg)
    })
}

/// Parse a subscription PDA with enhanced error messages
///
/// # Errors
/// Returns enhanced error with recovery suggestions if parsing fails
pub fn parse_subscription_pda(subscription_str: &str) -> Result<Pubkey> {
    Pubkey::from_str(subscription_str).map_err(|e| {
        let error_msg = format!(
            "Invalid subscription address: '{subscription_str}'\n\n\
             Subscription addresses must be base58-encoded Solana public keys (44 characters).\n\n\
             Did you mean to:\n  \
             • List subscriptions with: tally-merchant subscription list --plan <PLAN_PDA>\n  \
             • Check the address you copied\n\n\
             Example valid address: 9sPtLMn3uU1qYw6aS4oX9dQkPmN2wRbHkJgGbFtZvYe5\n\n\
             Original error: {e}"
        );

        anyhow!(error_msg)
    })
}

/// Wrap RPC errors with helpful recovery suggestions
///
/// # Errors
/// Returns enhanced error with network troubleshooting tips
#[must_use]
pub fn enhance_rpc_error(original_error: &anyhow::Error, rpc_url: &str) -> anyhow::Error {
    anyhow!(
        "{original_error}\n\n\
         RPC Connection Troubleshooting:\n  \
         • Check if the RPC endpoint is accessible: {rpc_url}\n  \
         • Try using a different RPC endpoint with --rpc-url\n  \
         • For devnet: https://api.devnet.solana.com\n  \
         • For mainnet: https://api.mainnet-beta.solana.com\n  \
         • Check your internet connection\n  \
         • The RPC endpoint may be rate-limiting your requests"
    )
}

/// Enhance account not found errors with suggestions
///
/// # Errors
/// Returns enhanced error with account troubleshooting tips
#[must_use]
pub fn enhance_account_not_found_error(
    account_type: &str,
    address: &Pubkey,
) -> anyhow::Error {
    match account_type {
        "merchant" => anyhow!(
            "Merchant account not found at address: {address}\n\n\
             This could mean:\n  \
             • The merchant account hasn't been created yet\n  \
             • You're using the wrong network (devnet vs mainnet)\n  \
             • The address is incorrect\n\n\
             To fix this:\n  \
             • Run 'tally-merchant init' to create a new merchant\n  \
             • Check you're on the correct network (--rpc-url)\n  \
             • Verify the merchant address with: tally-merchant config get merchant"
        ),
        "plan" => anyhow!(
            "Plan account not found at address: {address}\n\n\
             This could mean:\n  \
             • The plan hasn't been created yet\n  \
             • You're using the wrong network (devnet vs mainnet)\n  \
             • The plan may have been deactivated\n  \
             • The address is incorrect\n\n\
             To fix this:\n  \
             • List your plans with: tally-merchant plan list --merchant <MERCHANT_PDA>\n  \
             • Create a new plan with: tally-merchant plan create --help\n  \
             • Check you're on the correct network (--rpc-url)"
        ),
        "subscription" => anyhow!(
            "Subscription account not found at address: {address}\n\n\
             This could mean:\n  \
             • The subscription hasn't been created yet\n  \
             • You're using the wrong network (devnet vs mainnet)\n  \
             • The subscription may have been canceled\n  \
             • The address is incorrect\n\n\
             To fix this:\n  \
             • List subscriptions with: tally-merchant subscription list --plan <PLAN_PDA>\n  \
             • Check you're on the correct network (--rpc-url)"
        ),
        _ => anyhow!(
            "{account_type} account not found at address: {address}\n\n\
             Check:\n  \
             • The address is correct\n  \
             • You're on the right network (--rpc-url)\n  \
             • The account has been created"
        ),
    }
}

/// Enhance insufficient balance errors with actionable steps
///
/// # Errors
/// Returns enhanced error with funding instructions
#[must_use]
pub fn enhance_insufficient_balance_error(
    balance_sol: f64,
    required_sol: f64,
    network: &str,
) -> anyhow::Error {
    let shortage = required_sol - balance_sol;

    let funding_instructions = if network.contains("devnet") {
        "Get devnet SOL at: https://faucet.solana.com"
    } else if network.contains("mainnet") {
        "Get SOL from:\n  \
         • Centralized exchange (Coinbase, Binance, etc.)\n  \
         • Decentralized exchange (Jupiter, Raydium, etc.)\n  \
         • Crypto on-ramp service (MoonPay, Ramp, etc.)"
    } else {
        "Get SOL from a faucet or exchange"
    };

    anyhow!(
        "Insufficient SOL balance\n\n\
         Current balance: {balance_sol:.6} SOL\n\
         Required:        {required_sol:.6} SOL\n\
         Shortage:        {shortage:.6} SOL\n\n\
         {funding_instructions}\n\n\
         After funding, wait a few seconds for the balance to update,\n\
         then run your command again."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_merchant_pda_with_invalid_string() {
        let result = parse_merchant_pda("invalid", None);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Invalid merchant address"));
        assert!(error_message.contains("Did you mean to"));
        assert!(error_message.contains("tally-merchant init"));
    }

    #[test]
    fn test_parse_merchant_pda_with_saved_merchant() {
        let saved_merchant = Pubkey::new_unique();
        let result = parse_merchant_pda("invalid", Some(&saved_merchant));
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Use your saved merchant"));
        assert!(error_message.contains(&saved_merchant.to_string()));
    }

    #[test]
    fn test_parse_plan_pda_with_merchant() {
        let merchant = Pubkey::new_unique();
        let result = parse_plan_pda("invalid", Some(&merchant));
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Invalid plan address"));
        assert!(error_message.contains("plan list"));
        assert!(error_message.contains(&merchant.to_string()));
    }

    #[test]
    fn test_enhance_insufficient_balance_error() {
        let error = enhance_insufficient_balance_error(0.005, 0.01, "https://api.devnet.solana.com");
        let error_message = error.to_string();
        assert!(error_message.contains("Insufficient SOL balance"));
        assert!(error_message.contains("0.005000 SOL"));
        assert!(error_message.contains("0.010000 SOL"));
        assert!(error_message.contains("faucet.solana.com"));
    }

    #[test]
    fn test_enhance_account_not_found_error_merchant() {
        let address = Pubkey::new_unique();
        let error = enhance_account_not_found_error("merchant", &address);
        let error_message = error.to_string();
        assert!(error_message.contains("Merchant account not found"));
        assert!(error_message.contains("tally-merchant init"));
        assert!(error_message.contains(&address.to_string()));
    }
}
