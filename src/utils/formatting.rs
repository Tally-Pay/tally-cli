//! Output formatting utilities for the Tally CLI

use crate::config::TallyCliConfig;
use crate::utils::colors::{terminal_width, truncate_to_width, Theme};
use anyhow::{anyhow, Result};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};
use tally_sdk::solana_sdk::pubkey::Pubkey;

/// Plan information for display
#[derive(Debug)]
pub struct PlanInfo {
    pub address: Pubkey,
    pub plan_id: String,
    pub name: String,
    pub price_usdc: f64,
    pub period: String,
    pub grace_secs: u64,
    pub active: bool,
}

/// Subscription information for display
#[derive(Debug)]
pub struct SubscriptionInfo {
    pub address: Pubkey,
    pub plan: Pubkey,
    pub subscriber: Pubkey,
    pub next_renewal_ts: i64,
    pub active: bool,
    pub renewals: u32,
    pub created_ts: i64,
    pub last_amount: u64,
}

/// Format plans for human-readable output
#[must_use]
pub fn format_plans_human(plans: &[PlanInfo], merchant_pda: &Pubkey) -> String {
    use std::fmt::Write;

    if plans.is_empty() {
        return format!(
            "{} {}",
            Theme::warning("No plans found for merchant:"),
            Theme::dim(&merchant_pda.to_string())
        );
    }

    let mut output = String::new();
    writeln!(
        &mut output,
        "{} {}\n",
        Theme::header("Plans for merchant:"),
        Theme::highlight(&merchant_pda.to_string())
    )
    .unwrap();

    // Get terminal width for responsive layout
    let term_width = terminal_width();
    let available_width = term_width.saturating_sub(10);

    // Adjust column widths based on terminal size
    let (id_width, name_width, addr_width) = if available_width < 120 {
        // Narrow terminal - truncate columns
        (12, 15, 20)
    } else {
        // Wide terminal - full widths
        (15, 20, 44)
    };

    // Header with colors
    writeln!(
        &mut output,
        "{:<id_width$} {:<name_width$} {:<12} {:<15} {:<10} {:<8} {:<addr_width$}",
        Theme::header("Plan ID"),
        Theme::header("Name"),
        Theme::header("Price (USDC)"),
        Theme::header("Period"),
        Theme::header("Grace (s)"),
        Theme::header("Active"),
        Theme::header("Address"),
        id_width = id_width,
        name_width = name_width,
        addr_width = addr_width
    )
    .unwrap();
    output.push_str(&Theme::dim(&"-".repeat(available_width.min(140))).to_string());
    output.push('\n');

    // Data rows
    for plan in plans {
        let id_truncated = truncate_to_width(&plan.plan_id, id_width);
        let name_truncated = truncate_to_width(&plan.name, name_width);
        let addr_truncated = truncate_to_width(&plan.address.to_string(), addr_width);
        let active_text = if plan.active {
            Theme::active("Yes").to_string()
        } else {
            Theme::inactive("No").to_string()
        };

        writeln!(
            &mut output,
            "{:<id_width$} {:<name_width$} {:<12.6} {:<15} {:<10} {:<8} {}",
            id_truncated,
            name_truncated,
            plan.price_usdc,
            plan.period,
            plan.grace_secs,
            active_text,
            Theme::dim(&addr_truncated),
            id_width = id_width,
            name_width = name_width
        )
        .unwrap();
    }

    write!(
        &mut output,
        "\n{} {}",
        Theme::info("Total plans:"),
        Theme::value(&plans.len().to_string())
    )
    .unwrap();
    output
}

/// Format plans for JSON output
///
/// # Errors
///
/// Returns an error if JSON serialization fails
pub fn format_plans_json(plans: &[PlanInfo]) -> Result<String> {
    let json_plans: Vec<serde_json::Value> = plans
        .iter()
        .map(|plan| {
            serde_json::json!({
                "address": plan.address.to_string(),
                "plan_id": plan.plan_id,
                "name": plan.name,
                "price_usdc": plan.price_usdc,
                "period": plan.period,
                "grace_secs": plan.grace_secs,
                "active": plan.active
            })
        })
        .collect();

    serde_json::to_string_pretty(&json_plans)
        .map_err(|e| anyhow!("Failed to serialize plans to JSON: {e}"))
}

/// Format subscriptions for human-readable output
#[must_use]
pub fn format_subscriptions_human(
    subscriptions: &[SubscriptionInfo],
    plan_pda: &Pubkey,
    config: &TallyCliConfig,
) -> String {
    use std::fmt::Write;

    if subscriptions.is_empty() {
        return format!(
            "{} {}",
            Theme::warning("No subscriptions found for plan:"),
            Theme::dim(&plan_pda.to_string())
        );
    }

    let mut output = String::new();
    writeln!(
        &mut output,
        "{} {}\n",
        Theme::header("Subscriptions for plan:"),
        Theme::highlight(&plan_pda.to_string())
    )
    .unwrap();

    // Get terminal width for responsive layout
    let term_width = terminal_width();
    let available_width = term_width.saturating_sub(10);

    // Adjust column widths based on terminal size
    let (subscriber_width, addr_width) = if available_width < 120 {
        // Narrow terminal - truncate columns
        (20, 20)
    } else {
        // Wide terminal - full widths
        (44, 44)
    };

    // Header with colors
    writeln!(
        &mut output,
        "{:<subscriber_width$} {:<8} {:<9} {:<20} {:<20} {:<12} {:<addr_width$}",
        Theme::header("Subscriber"),
        Theme::header("Status"),
        Theme::header("Renewals"),
        Theme::header("Next Renewal"),
        Theme::header("Created"),
        Theme::header("Last Amount"),
        Theme::header("Address"),
        subscriber_width = subscriber_width,
        addr_width = addr_width
    )
    .unwrap();
    output.push_str(&Theme::dim(&"-".repeat(available_width.min(175))).to_string());
    output.push('\n');

    // Data rows
    for sub in subscriptions {
        let next_renewal = format_timestamp(sub.next_renewal_ts);
        let created = format_timestamp(sub.created_ts);
        let last_amount_usdc = config.format_usdc(sub.last_amount);

        let subscriber_truncated = truncate_to_width(&sub.subscriber.to_string(), subscriber_width);
        let addr_truncated = truncate_to_width(&sub.address.to_string(), addr_width);
        let status_text = if sub.active {
            Theme::active("Active").to_string()
        } else {
            Theme::inactive("Inactive").to_string()
        };

        writeln!(
            &mut output,
            "{:<subscriber_width$} {:<8} {:<9} {:<20} {:<20} {:<12.6} {}",
            subscriber_truncated,
            status_text,
            sub.renewals,
            next_renewal,
            created,
            last_amount_usdc,
            Theme::dim(&addr_truncated),
            subscriber_width = subscriber_width
        )
        .unwrap();
    }

    write!(
        &mut output,
        "\n{} {}",
        Theme::info("Total subscriptions:"),
        Theme::value(&subscriptions.len().to_string())
    )
    .unwrap();
    output
}

/// Format subscriptions for JSON output
///
/// # Errors
///
/// Returns an error if JSON serialization fails
pub fn format_subscriptions_json(
    subscriptions: &[SubscriptionInfo],
    config: &TallyCliConfig,
) -> Result<String> {
    let json_subscriptions: Vec<serde_json::Value> = subscriptions
        .iter()
        .map(|sub| {
            serde_json::json!({
                "address": sub.address.to_string(),
                "plan": sub.plan.to_string(),
                "subscriber": sub.subscriber.to_string(),
                "next_renewal_ts": sub.next_renewal_ts,
                "next_renewal": format_timestamp(sub.next_renewal_ts),
                "active": sub.active,
                "renewals": sub.renewals,
                "created_ts": sub.created_ts,
                "created": format_timestamp(sub.created_ts),
                "last_amount": sub.last_amount,
                "last_amount_usdc": config.format_usdc(sub.last_amount)
            })
        })
        .collect();

    serde_json::to_string_pretty(&json_subscriptions)
        .map_err(|e| anyhow!("Failed to serialize subscriptions to JSON: {e}"))
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
