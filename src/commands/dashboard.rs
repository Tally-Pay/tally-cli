//! Dashboard commands implementation

use crate::config::TallyCliConfig;
use anyhow::{Context, Result};
use clap::ValueEnum;
use std::fmt::Write as _;
use std::str::FromStr;
use tally_sdk::solana_sdk::pubkey::Pubkey;
use tally_sdk::{DashboardClient, SimpleTallyClient, UsdcAmount};

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Csv,
}

/// Wrapper function for backwards compatibility with main.rs
///
/// # Errors
/// Returns error if dashboard operation fails or merchant/plan not found
pub fn execute<T: std::fmt::Debug + Send + Sync>(
    _tally_client: &SimpleTallyClient,
    command: &T,
    output_format: &OutputFormat,
    rpc_url: &str,
    config: &TallyCliConfig,
) -> Result<String> {
    execute_dashboard_command(command, output_format, rpc_url, config)
}

/// Execute dashboard command with proper routing
///
/// # Errors
/// Returns error if dashboard operation fails or merchant/plan not found
pub fn execute_dashboard_command<T: std::fmt::Debug + Send + Sync>(
    command: &T,
    output_format: &OutputFormat,
    rpc_url: &str,
    config: &TallyCliConfig,
) -> Result<String> {
    // Create dashboard client
    let dashboard_client =
        DashboardClient::new(rpc_url).context("Failed to create dashboard client")?;

    // Route to appropriate handler based on command type
    // Since we don't have access to the actual DashboardCommands type here,
    // we'll use the command debug string to route
    let command_str = format!("{command:?}");

    if command_str.contains("Overview") {
        // Extract merchant address from command
        extract_and_execute_overview(&dashboard_client, &command_str, output_format, config)
    } else if command_str.contains("Analytics") {
        // Extract plan address from command
        extract_and_execute_analytics(&dashboard_client, &command_str, output_format, config)
    } else if command_str.contains("Events") {
        // Extract merchant address and optional since timestamp
        extract_and_execute_events(&dashboard_client, &command_str, output_format, config)
    } else if command_str.contains("Subscriptions") {
        // Extract merchant address and active_only flag
        extract_and_execute_subscriptions(&dashboard_client, &command_str, output_format, config)
    } else {
        Err(anyhow::anyhow!("Unknown dashboard command"))
    }
}

/// Execute the Overview command
fn extract_and_execute_overview(
    dashboard_client: &DashboardClient,
    command_str: &str,
    output_format: &OutputFormat,
    config: &TallyCliConfig,
) -> Result<String> {
    // Parse merchant address from command string
    let merchant_str = command_str
        .split("merchant:")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.trim_matches(|c| c == '"' || c == ',').split(',').next())
        // Strip Option wrapper if present (e.g., "Some(\"address\")" -> "address")
        .map(|s| {
            s.trim_start_matches("Some(")
                .trim_end_matches(')')
                .trim_matches('"')
        })
        .context("Failed to extract merchant address from command")?;

    let merchant = Pubkey::from_str(merchant_str)
        .context(format!("Invalid merchant address: {merchant_str}"))?;

    // Get overview data
    let overview = dashboard_client
        .get_payee_overview(&merchant)
        .context("Failed to fetch merchant overview")?;

    // Format output
    match output_format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&overview)?;
            Ok(json)
        }
        OutputFormat::Csv => {
            // CSV output for overview statistics
            let mut wtr = csv::Writer::from_writer(vec![]);

            // Write header
            wtr.write_record(["metric", "value"])?;

            // Write metrics
            wtr.write_record(["merchant_address", &merchant.to_string()])?;
            wtr.write_record(["merchant_authority", &overview.payee_authority.to_string()])?;
            wtr.write_record(["usdc_mint", &overview.usdc_mint.to_string()])?;
            wtr.write_record([
                "total_revenue_usdc",
                &UsdcAmount::from_microlamports(overview.total_revenue).to_string(),
            ])?;
            wtr.write_record([
                "monthly_revenue_usdc",
                &UsdcAmount::from_microlamports(overview.monthly_revenue).to_string(),
            ])?;
            wtr.write_record([
                "average_revenue_per_user_usdc",
                &config.format_usdc(overview.average_revenue_per_payer),
            ])?;
            wtr.write_record(["total_plans", &overview.total_payment_terms.to_string()])?;
            wtr.write_record([
                "active_subscriptions",
                &overview.active_agreements.to_string(),
            ])?;
            wtr.write_record([
                "inactive_subscriptions",
                &overview.inactive_agreements.to_string(),
            ])?;
            wtr.write_record([
                "churn_rate_percent",
                &format!("{:.2}", overview.churn_rate()),
            ])?;
            wtr.write_record([
                "monthly_new_subscriptions",
                &overview.monthly_new_agreements.to_string(),
            ])?;
            wtr.write_record([
                "monthly_canceled_subscriptions",
                &overview.monthly_paused_agreements.to_string(),
            ])?;

            let data = String::from_utf8(wtr.into_inner()?)?;
            Ok(data)
        }
        OutputFormat::Human => {
            let mut output = format!("\nMerchant Dashboard Overview - {merchant}\n");
            output.push_str(&"=".repeat(70));
            output.push('\n');

            output.push_str("\nRevenue Statistics:\n");
            writeln!(
                output,
                "  Total Revenue:        {} USDC",
                UsdcAmount::from_microlamports(overview.total_revenue)
            )?;
            writeln!(
                output,
                "  Monthly Revenue:      {} USDC",
                UsdcAmount::from_microlamports(overview.monthly_revenue)
            )?;
            writeln!(
                output,
                "  Avg Revenue per User: {} USDC",
                UsdcAmount::from_microlamports(overview.average_revenue_per_payer)
            )?;

            output.push_str("\nSubscription Statistics:\n");
            writeln!(
                output,
                "  Total Plans:          {}",
                overview.total_payment_terms
            )?;
            writeln!(
                output,
                "  Active Subscriptions: {}",
                overview.active_agreements
            )?;
            writeln!(
                output,
                "  Inactive Subscriptions: {}",
                overview.inactive_agreements
            )?;
            writeln!(
                output,
                "  Churn Rate:           {:.2}%",
                overview.churn_rate()
            )?;

            output.push_str("\nMonthly Growth:\n");
            writeln!(
                output,
                "  New Subscriptions:    {}",
                overview.monthly_new_agreements
            )?;
            writeln!(
                output,
                "  PaymentAgreementPaused Subscriptions: {}",
                overview.monthly_paused_agreements
            )?;

            output.push_str("\nConfiguration:\n");
            writeln!(
                output,
                "  Merchant Authority:   {}",
                overview.payee_authority
            )?;
            writeln!(output, "  USDC Mint:            {}", overview.usdc_mint)?;

            Ok(output)
        }
    }
}

/// Execute the Analytics command
fn extract_and_execute_analytics(
    dashboard_client: &DashboardClient,
    command_str: &str,
    output_format: &OutputFormat,
    config: &TallyCliConfig,
) -> Result<String> {
    // Parse plan address from command string
    let plan_str = command_str
        .split("plan:")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.trim_matches(|c| c == '"' || c == ',').split(',').next())
        .context("Failed to extract plan address from command")?;

    let plan = Pubkey::from_str(plan_str).context(format!("Invalid plan address: {plan_str}"))?;

    // Get plan analytics
    let analytics = dashboard_client
        .get_payment_terms_analytics(&plan)
        .context("Failed to fetch plan analytics")?;

    // Format output
    match output_format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&analytics)?;
            Ok(json)
        }
        OutputFormat::Csv => {
            // CSV output for plan analytics
            let plan_id_str = String::from_utf8(analytics.payment_terms.terms_id.to_vec())
                .unwrap_or_else(|_| format!("{:?}", analytics.payment_terms.terms_id));

            let mut wtr = csv::Writer::from_writer(vec![]);

            // Write header
            wtr.write_record(["metric", "value"])?;

            // Write metrics
            wtr.write_record(["plan_id", &plan_id_str])?;
            wtr.write_record(["plan_address", &analytics.payment_terms_address.to_string()])?;
            wtr.write_record([
                "price_usdc",
                &config.format_usdc(analytics.payment_terms.amount_usdc),
            ])?;
            wtr.write_record([
                "period_seconds",
                &analytics.payment_terms.period_secs.to_string(),
            ])?;
            wtr.write_record(["active", "true"])?;
            wtr.write_record([
                "total_revenue_usdc",
                &UsdcAmount::from_microlamports(analytics.total_revenue).to_string(),
            ])?;
            wtr.write_record([
                "monthly_revenue_usdc",
                &UsdcAmount::from_microlamports(analytics.monthly_revenue).to_string(),
            ])?;
            wtr.write_record(["active_count", &analytics.active_count.to_string()])?;
            wtr.write_record(["inactive_count", &analytics.inactive_count.to_string()])?;
            wtr.write_record([
                "total_subscriptions",
                &analytics.total_agreements().to_string(),
            ])?;
            wtr.write_record([
                "churn_rate_percent",
                &format!("{:.2}", analytics.churn_rate()),
            ])?;
            wtr.write_record([
                "average_duration_days",
                &format!("{:.1}", analytics.average_duration_days),
            ])?;
            wtr.write_record([
                "monthly_new_subscriptions",
                &analytics.monthly_new_agreements.to_string(),
            ])?;
            wtr.write_record([
                "monthly_canceled_subscriptions",
                &analytics.monthly_paused_agreements.to_string(),
            ])?;
            wtr.write_record([
                "monthly_growth_rate_percent",
                &format!("{:.2}", analytics.monthly_growth_rate()),
            ])?;
            if let Some(conversion_rate) = analytics.conversion_rate {
                wtr.write_record(["conversion_rate_percent", &format!("{conversion_rate:.2}")])?;
            }

            let data = String::from_utf8(wtr.into_inner()?)?;
            Ok(data)
        }
        OutputFormat::Human => {
            // Convert plan_id bytes to string
            let plan_id_str = String::from_utf8(analytics.payment_terms.terms_id.to_vec())
                .unwrap_or_else(|_| format!("{:?}", analytics.payment_terms.terms_id));

            let mut output = format!("\nPlan Analytics - {plan_id_str}\n");
            output.push_str(&"=".repeat(70));
            output.push('\n');

            output.push_str("\nPlan Information:\n");
            writeln!(output, "  Plan ID:              {plan_id_str}")?;
            writeln!(
                output,
                "  Plan Address:         {}",
                analytics.payment_terms_address
            )?;
            writeln!(
                output,
                "  Price:                {} USDC",
                UsdcAmount::from_microlamports(analytics.payment_terms.amount_usdc)
            )?;
            writeln!(
                output,
                "  Period:               {} seconds",
                analytics.payment_terms.period_secs
            )?;
            writeln!(output, "  Active:               true")?;

            output.push_str("\nRevenue Statistics:\n");
            writeln!(
                output,
                "  Total Revenue:        {} USDC",
                UsdcAmount::from_microlamports(analytics.total_revenue)
            )?;
            writeln!(
                output,
                "  Monthly Revenue:      {} USDC",
                UsdcAmount::from_microlamports(analytics.monthly_revenue)
            )?;

            output.push_str("\nSubscription Statistics:\n");
            writeln!(output, "  Active Count:         {}", analytics.active_count)?;
            writeln!(
                output,
                "  Inactive Count:       {}",
                analytics.inactive_count
            )?;
            writeln!(
                output,
                "  Total Subscriptions:  {}",
                analytics.total_agreements()
            )?;
            writeln!(
                output,
                "  Churn Rate:           {:.2}%",
                analytics.churn_rate()
            )?;
            writeln!(
                output,
                "  Avg Duration:         {:.1} days",
                analytics.average_duration_days
            )?;

            output.push_str("\nMonthly Growth:\n");
            writeln!(
                output,
                "  New Subscriptions:    {}",
                analytics.monthly_new_agreements
            )?;
            writeln!(
                output,
                "  PaymentAgreementPaused Subscriptions: {}",
                analytics.monthly_paused_agreements
            )?;
            writeln!(
                output,
                "  Growth Rate:          {:.2}%",
                analytics.monthly_growth_rate()
            )?;

            if let Some(conversion_rate) = analytics.conversion_rate {
                writeln!(output, "  Conversion Rate:      {conversion_rate:.2}%")?;
            }

            Ok(output)
        }
    }
}

/// Execute the Events command
fn extract_and_execute_events(
    dashboard_client: &DashboardClient,
    command_str: &str,
    _output_format: &OutputFormat,
    config: &TallyCliConfig,
) -> Result<String> {
    // Parse merchant address from command string
    let merchant_str = command_str
        .split("merchant:")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.trim_matches(|c| c == '"' || c == ',').split(',').next())
        // Strip Option wrapper if present (e.g., "Some(\"address\")" -> "address")
        .map(|s| {
            s.trim_start_matches("Some(")
                .trim_end_matches(')')
                .trim_matches('"')
        })
        .context("Failed to extract merchant address from command")?;

    let merchant = Pubkey::from_str(merchant_str)
        .context(format!("Invalid merchant address: {merchant_str}"))?;

    // Parse optional since timestamp (defaults to 1 hour ago)
    let current_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0);

    let since_timestamp = if command_str.contains("since: Some(") {
        command_str
            .split("since: Some(")
            .nth(1)
            .and_then(|s| s.split(')').next())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(current_timestamp - config.default_events_lookback_secs)
    } else {
        current_timestamp - config.default_events_lookback_secs
    };

    // Get recent events
    let events = dashboard_client
        .poll_recent_events(&merchant, since_timestamp)
        .context("Failed to fetch recent events")?;

    // Format output
    let mut output = format!("\nRecent Events for Merchant: {merchant}\n");
    output.push_str(&"=".repeat(70));
    output.push('\n');
    writeln!(
        output,
        "\nShowing events since timestamp: {since_timestamp}\n"
    )?;

    if events.is_empty() {
        output.push_str("No events found in the specified time period.\n");
    } else {
        writeln!(output, "Total Events: {}\n", events.len())?;

        for event in &events {
            writeln!(output, "Event: {:?}", event.event_type)?;
            writeln!(output, "  Timestamp: {}", event.timestamp)?;

            if let Some(subscriber) = event.payer {
                writeln!(output, "  Subscriber: {subscriber}")?;
            }
            if let Some(plan) = event.payment_terms_address {
                writeln!(output, "  Plan: {plan}")?;
            }
            if let Some(amount) = event.amount {
                writeln!(output, "  Amount: {} USDC", UsdcAmount::from_microlamports(amount))?;
            }
            if let Some(sig) = &event.transaction_signature {
                writeln!(output, "  Signature: {sig}")?;
            }

            output.push('\n');
        }
    }

    Ok(output)
}

/// Execute the Subscriptions command
fn extract_and_execute_subscriptions(
    dashboard_client: &DashboardClient,
    command_str: &str,
    output_format: &OutputFormat,
    config: &TallyCliConfig,
) -> Result<String> {
    // Parse merchant address from command string
    let merchant_str = command_str
        .split("merchant:")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.trim_matches(|c| c == '"' || c == ',').split(',').next())
        // Strip Option wrapper if present (e.g., "Some(\"address\")" -> "address")
        .map(|s| {
            s.trim_start_matches("Some(")
                .trim_end_matches(')')
                .trim_matches('"')
        })
        .context("Failed to extract merchant address from command")?;

    let merchant = Pubkey::from_str(merchant_str)
        .context(format!("Invalid merchant address: {merchant_str}"))?;

    // Parse active_only flag
    let active_only = command_str.contains("active_only: true");

    // Get subscriptions
    let mut subscriptions = dashboard_client
        .get_live_agreements(&merchant)
        .context("Failed to fetch subscriptions")?;

    // Filter if active_only
    if active_only {
        subscriptions.retain(|sub| sub.payment_agreement.active);
    }

    // Format output
    match output_format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&subscriptions)?;
            Ok(json)
        }
        OutputFormat::Csv => {
            // CSV output for subscriptions
            let mut wtr = csv::Writer::from_writer(vec![]);

            // Write header
            wtr.write_record([
                "subscriber",
                "plan_id",
                "plan_address",
                "status",
                "active",
                "renewals",
                "total_paid_usdc",
                "price_usdc",
                "period_seconds",
                "next_renewal_timestamp",
            ])?;

            // Write data rows
            for sub in &subscriptions {
                let plan_id_str = String::from_utf8(sub.payment_terms.terms_id.to_vec())
                    .unwrap_or_else(|_| format!("{:?}", sub.payment_terms.terms_id));

                wtr.write_record([
                    sub.payment_agreement.payer.to_string(),
                    plan_id_str,
                    sub.payment_agreement.payment_terms.to_string(),
                    format!("{:?}", sub.status),
                    sub.payment_agreement.active.to_string(),
                    sub.payment_agreement.payment_count.to_string(),
                    UsdcAmount::from_microlamports(sub.total_paid).to_string(),
                    config.format_usdc(sub.payment_terms.amount_usdc),
                    sub.payment_terms.period_secs.to_string(),
                    sub.payment_agreement.next_payment_ts.to_string(),
                ])?;
            }

            let data = String::from_utf8(wtr.into_inner()?)?;
            Ok(data)
        }
        OutputFormat::Human => {
            let mut output = format!("\nSubscriptions for Merchant: {merchant}\n");
            output.push_str(&"=".repeat(100));
            output.push('\n');
            write!(output, "\n{} subscriptions found", subscriptions.len())?;
            if active_only {
                output.push_str(" (active only)");
            }
            output.push_str("\n\n");

            if subscriptions.is_empty() {
                output.push_str("No subscriptions found.\n");
            } else {
                // Header
                writeln!(
                    output,
                    "{:<45} {:<45} {:<12} {:<12} {:<12}",
                    "Subscriber", "Plan", "Status", "Renewals", "Total Paid"
                )?;
                output.push_str(&"-".repeat(100));
                output.push('\n');

                // Data rows
                for sub in &subscriptions {
                    let subscriber_str = sub.payment_agreement.payer.to_string();
                    let plan_id_str = String::from_utf8(sub.payment_terms.terms_id.to_vec())
                        .unwrap_or_else(|_| format!("{:?}", sub.payment_terms.terms_id));
                    let status_str = format!("{:?}", sub.status);
                    let total_paid_str = format!("{} USDC", UsdcAmount::from_microlamports(sub.total_paid));

                    writeln!(
                        output,
                        "{:<45} {:<45} {:<12} {:<12} {:<12}",
                        truncate_string(&subscriber_str, 44),
                        truncate_string(&plan_id_str, 44),
                        status_str,
                        sub.payment_agreement.payment_count,
                        total_paid_str
                    )?;
                }
            }

            Ok(output)
        }
    }
}

/// Truncate string to max length with ellipsis if needed
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is..."
        );
        assert_eq!(truncate_string("exactly10!", 10), "exactly10!");
    }
}
