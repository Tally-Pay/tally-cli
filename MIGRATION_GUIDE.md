# Tally SDK Migration Guide

**Last Updated:** November 10, 2025
**Audience:** Internal developers (CLI tools, services, keeper infrastructure)

---

## Overview

This guide covers recent API improvements made to the Tally SDK before the v1.0 launch. The changes focus on **type safety**, **API simplification**, and **developer experience**.

### What Changed

| Area | Change Type | Impact |
|------|-------------|--------|
| **Type Safety** | New newtype wrappers | HIGH - Required for most operations |
| **PDA API** | Simplified from 20 → 5 functions | HIGH - API surface reduced 75% |
| **Builders** | Typestate pattern | MEDIUM - Compile-time validation |
| **Errors** | Specific error variants | MEDIUM - Better error handling |
| **Event Parsing** | Enhanced error reporting | LOW - Opt-in improvements |

---

## Quick Start: What You Need to Do

### 1. Update Imports

```rust
// Add new type imports
use tally_sdk::{
    UsdcAmount,      // NEW
    BasisPoints,     // NEW
    PaymentPeriod,   // NEW
    TermsId,         // NEW
};
```

### 2. Update Numeric Values

**Before:**
```rust
let amount = 10_000_000_u64;  // What is this?
let fee = 50_u16;             // 50 what?
let period = 2_592_000_u64;   // Seconds?
```

**After:**
```rust
let amount = UsdcAmount::from_usdc(10.0);      // $10 USDC (clear!)
let fee = BasisPoints::new(50)?;                // 0.50% (validated!)
let period = PaymentPeriod::days(30)?;          // 30 days (readable!)
```

### 3. Update PDA Calls

**Before:**
```rust
// Which function do I use?
let (payee_pda, bump) = pda::payee(&authority)?;
let payee_pda = pda::payee_address(&authority)?;
```

**After:**
```rust
// Simple, consistent API
let payee_pda = pda::payee(&authority)?;

// Need bump? Use .with_bump()
let (payee_pda, bump) = pda::payee(&authority)?.with_bump();
```

---

## Detailed Migration Steps

### Step 1: Replace Primitive Types with Newtypes

#### USDC Amounts

**Old Code:**
```rust
// CLI: Create payment terms
let amount_usdc: u64 = 10_000_000;  // $10 in microlamports

// Service: Calculate fees
let payment_amount = agreement.last_amount;
let fee = (payment_amount as u128 * 250 / 10_000) as u64;  // Manual calculation
```

**New Code:**
```rust
use tally_sdk::UsdcAmount;

// CLI: Create payment terms
let amount = UsdcAmount::from_usdc(10.0);  // Clear and type-safe

// Service: Calculate fees
let payment_amount = UsdcAmount::from_microlamports(agreement.last_amount);
let fee_bps = BasisPoints::new(250)?;  // 2.5%
let fee = fee_bps.apply_to(payment_amount);  // Type-safe calculation

// Display
println!("Payment: {}, Fee: {}", payment_amount, fee);
// Output: "Payment: $10.00, Fee: $0.25"
```

**Benefits:**
- ✅ No more manual unit conversions
- ✅ Self-documenting code
- ✅ Built-in formatting
- ✅ Type-safe arithmetic

#### Basis Points (Fees)

**Old Code:**
```rust
// CLI: Initialize payee
let payee_fee_bps: u16 = 50;  // Is this 50% or 0.5%?

// Validation scattered everywhere
if payee_fee_bps > 10_000 {
    return Err("Fee exceeds 100%".into());
}
```

**New Code:**
```rust
use tally_sdk::BasisPoints;

// CLI: Initialize payee
let payee_fee = BasisPoints::new(50)?;  // Validated at construction!

// From percentage
let platform_fee = BasisPoints::from_percentage(2.5)?;  // 2.5% → 250 bps

// Display
println!("Fee: {}", payee_fee);  // Output: "0.50%"
```

**Benefits:**
- ✅ Validation at construction (can't exceed 100%)
- ✅ Clear percentage vs basis points distinction
- ✅ Human-readable display

#### Payment Periods

**Old Code:**
```rust
// CLI: Create payment terms
let period_secs: u64 = 30 * 24 * 60 * 60;  // 30 days
let grace_secs: u64 = 3 * 24 * 60 * 60;    // 3 days

// Manual validation
if period_secs < 86_400 {
    return Err("Period must be at least 24 hours".into());
}
```

**New Code:**
```rust
use tally_sdk::PaymentPeriod;

// CLI: Create payment terms
let period = PaymentPeriod::days(30)?;      // Validated!
let grace = PaymentPeriod::days(3)?;        // Clear intent!

// From seconds (for custom periods)
let custom = PaymentPeriod::from_seconds(172_800)?;  // 2 days

// Display
println!("Period: {}", period);  // Output: "30 days"
```

**Benefits:**
- ✅ Automatic validation (minimum 24 hours)
- ✅ Readable duration construction
- ✅ Human-friendly display

#### Terms Identifiers

**Old Code:**
```rust
// CLI: Create payment terms
let terms_id = "premium_monthly";  // No validation!

// Easy to typo
let terms_pda = pda::payment_terms_from_string(&payee, "premium-monthly")?;  // Different!
let terms_pda2 = pda::payment_terms_from_string(&payee, "premium_monthly")?; // Different PDA!
```

**New Code:**
```rust
use tally_sdk::TermsId;

// CLI: Create payment terms
let terms_id = TermsId::new("premium_monthly")?;  // Validated!

// Catches errors early
let invalid = TermsId::new("premium monthly");  // Error: spaces not allowed
let invalid2 = TermsId::new("plan@2024");       // Error: special chars not allowed
let invalid3 = TermsId::new("");                // Error: empty not allowed

// Use with PDA functions
let terms_pda = pda::payment_terms(&payee, &terms_id)?;
```

**Validation Rules:**
- ✅ Non-empty
- ✅ Max 32 bytes
- ✅ Alphanumeric + underscores + hyphens only

---

### Step 2: Simplify PDA Computations

The PDA API has been dramatically simplified from 20 functions to 5 core functions with optional modifiers.

#### Payee PDAs

**Old Code:**
```rust
// Too many choices!
let (payee_pda, bump) = pda::payee(&authority)?;           // Returns tuple
let payee_pda = pda::payee_address(&authority)?;            // Returns Pubkey only
let (payee_pda, bump) = pda::payee_with_program_id(&authority, &program_id);  // Custom program
let payee_pda = pda::payee_address_with_program_id(&authority, &program_id);
```

**New Code:**
```rust
// Simple, consistent API
let payee_pda = pda::payee(&authority)?;  // Returns Pubkey (95% of cases)

// Advanced: Need bump?
let (payee_pda, bump) = pda::payee(&authority)?.with_bump();

// Advanced: Custom program ID?
let payee_pda = pda::payee(&authority)?.with_program_id(&custom_program_id);

// Advanced: Both bump and custom program?
let (payee_pda, bump) = pda::payee(&authority)?
    .with_program_id(&custom_program_id)
    .with_bump();
```

#### Payment Terms PDAs

**Old Code:**
```rust
// String vs bytes confusion
let terms_pda = pda::payment_terms_from_string(&payee, "premium")?;  // String
let terms_pda = pda::payment_terms(&payee, b"premium")?;              // Bytes
let terms_pda = pda::payment_terms_address(&payee, b"premium")?;      // Address only
```

**New Code:**
```rust
use tally_sdk::TermsId;

// Type-safe, validated
let terms_id = TermsId::new("premium")?;
let terms_pda = pda::payment_terms(&payee, &terms_id)?;

// Or inline (also validated)
let terms_pda = pda::payment_terms(&payee, &TermsId::new("premium")?)?;
```

#### Payment Agreement PDAs

**Old Code:**
```rust
let (agreement_pda, bump) = pda::payment_agreement(&terms_pda, &payer)?;
let agreement_pda = pda::payment_agreement_address(&terms_pda, &payer)?;
```

**New Code:**
```rust
// Consistent with other PDAs
let agreement_pda = pda::payment_agreement(&terms_pda, &payer)?;

// Need bump?
let (agreement_pda, bump) = pda::payment_agreement(&terms_pda, &payer)?.with_bump();
```

#### Config and Delegate PDAs

**Old Code:**
```rust
let (config_pda, bump) = pda::config()?;
let config_pda = pda::config_address()?;

let (delegate_pda, bump) = pda::delegate()?;
let delegate_pda = pda::delegate_address()?;
```

**New Code:**
```rust
// Simple, consistent
let config_pda = pda::config()?;
let delegate_pda = pda::delegate()?;

// With bump if needed
let (config_pda, bump) = pda::config()?.with_bump();
let (delegate_pda, bump) = pda::delegate()?.with_bump();
```

---

### Step 3: Update Transaction Builders

Builders now use typestate pattern for compile-time validation of required parameters.

#### Creating Payment Terms

**Old Code:**
```rust
use tally_sdk::{CreatePaymentTermsArgs, create_payment_terms};

let args = CreatePaymentTermsArgs {
    terms_id: "premium".to_string(),
    amount_usdc: 10_000_000,      // Raw u64
    period_secs: 2_592_000,       // Raw u64
    grace_secs: 259_200,          // Raw u64
    name: "Premium Monthly".to_string(),
};

// Can forget required parameters - runtime error!
let (terms_pda, sig) = client.create_payment_terms(&authority, args)?;
```

**New Code:**
```rust
use tally_sdk::{TermsId, UsdcAmount, PaymentPeriod};

// Type-safe, validated, compile-time checked
let (terms_pda, sig) = client.create_payment_terms(
    TermsId::new("premium")?,
    UsdcAmount::from_usdc(10.0),
    PaymentPeriod::days(30)?,
    PaymentPeriod::days(3)?,   // grace period
    "Premium Monthly",
)?;

// Compile error if you forget parameters!
```

#### Starting Payment Agreements

**Old Code:**
```rust
use tally_sdk::{StartAgreementArgs, start_agreement};

let args = StartAgreementArgs {
    allowance_periods: 3,
};

// What are the units of allowance_periods?
let (agreement_pda, sig) = client.start_agreement(&payer, &terms_pda, args)?;
```

**New Code:**
```rust
// Clear and explicit
let (agreement_pda, sig) = client.start_agreement(
    &payer,
    &terms_pda,
    3,  // allowance periods (multiplier for delegate approval)
)?;

// Or use constants for common values
use tally_sdk::constants::DEFAULT_ALLOWANCE_PERIODS;
let (agreement_pda, sig) = client.start_agreement(
    &payer,
    &terms_pda,
    DEFAULT_ALLOWANCE_PERIODS,
)?;
```

---

### Step 4: Improve Error Handling

**Old Code:**
```rust
use tally_sdk::TallyError;

// Generic errors - hard to handle
match result {
    Err(TallyError::Generic(msg)) => {
        // What kind of error is this?
        eprintln!("Error: {}", msg);
    }
    _ => {}
}
```

**New Code:**
```rust
use tally_sdk::TallyError;

// Specific error variants - easy to handle
match result {
    Err(TallyError::InvalidParameter { parameter, reason }) => {
        eprintln!("Invalid {}: {}", parameter, reason);
        // Can handle specific parameter errors
    }
    Err(TallyError::MissingParameter { parameter }) => {
        eprintln!("Missing required parameter: {}", parameter);
    }
    Err(TallyError::PayeeNotFound) => {
        eprintln!("Payee account not initialized");
        // Suggest initialization
    }
    Err(TallyError::PaymentTermsNotFound) => {
        eprintln!("Payment terms not found");
        // Suggest creation
    }
    _ => {}
}
```

**New Error Variants:**
- `MissingParameter { parameter: &'static str }`
- `InvalidParameter { parameter: &'static str, reason: String }`
- `ConfigError(String)`

---

### Step 5: Enhanced Event Parsing

**Old Code:**
```rust
use tally_sdk::parse_events_from_logs;

let events = parse_events_from_logs(&logs);  // Vec<TallyEvent>

// No way to know if parsing failed or there were no events
if events.is_empty() {
    // Did parsing fail? Or are there just no events?
}
```

**New Code:**
```rust
use tally_sdk::parse_events_from_logs;

let result = parse_events_from_logs(&logs);  // EventParseResult

// Clear distinction between success and failure
println!("Successfully parsed {} events", result.events.len());

if !result.errors.is_empty() {
    eprintln!("Parsing errors:");
    for error in &result.errors {
        eprintln!("  Line {}: {}", error.log_index, error.error);
    }
}

// Handle unknown events (forward compatibility)
for event in &result.events {
    match event {
        TallyEvent::PaymentExecuted(e) => { /* ... */ }
        TallyEvent::Unknown { discriminator, data } => {
            eprintln!("Unknown event: {:?}", discriminator);
            // Log for future investigation
        }
        _ => {}
    }
}
```

---

## Complete Migration Examples

### Example 1: CLI - Initialize Payee and Create Terms

**Old Code:**
```rust
// cli/src/commands/init.rs
use tally_sdk::{
    SimpleTallySigningClient, InitPayeeArgs, CreatePaymentTermsArgs,
    load_keypair, pda, ata,
};

pub fn init_payee_and_terms() -> Result<()> {
    let authority = load_keypair(None)?;
    let client = SimpleTallySigningClient::from_keypair_path("https://api.devnet.solana.com", None)?;

    let usdc_mint = get_usdc_mint()?;
    let treasury_ata = ata::get_associated_token_address_with_program(
        &authority.pubkey(),
        &usdc_mint,
        TokenProgram::Token,
    )?;

    // Initialize payee
    let (payee_pda, _sig) = client.init_payee(
        &authority,
        &usdc_mint,
        &treasury_ata,
    )?;

    // Create payment terms
    let terms_args = CreatePaymentTermsArgs {
        terms_id: "premium_monthly".to_string(),
        amount_usdc: 10_000_000,   // $10
        period_secs: 2_592_000,     // 30 days
        grace_secs: 259_200,        // 3 days
        name: "Premium Monthly".to_string(),
    };

    let (terms_pda, _sig) = client.create_payment_terms(&authority, terms_args)?;

    println!("Payee PDA: {}", payee_pda);
    println!("Terms PDA: {}", terms_pda);

    Ok(())
}
```

**New Code:**
```rust
// cli/src/commands/init.rs
use tally_sdk::{
    SimpleTallySigningClient, TermsId, UsdcAmount, PaymentPeriod,
    load_keypair, pda, ata,
};

pub fn init_payee_and_terms() -> Result<()> {
    let authority = load_keypair(None)?;
    let client = SimpleTallySigningClient::from_keypair_path("https://api.devnet.solana.com", None)?;

    let usdc_mint = get_usdc_mint()?;
    let treasury_ata = ata::get_associated_token_address_with_program(
        &authority.pubkey(),
        &usdc_mint,
        TokenProgram::Token,
    )?;

    // Initialize payee (same as before)
    let (payee_pda, _sig) = client.init_payee(
        &authority,
        &usdc_mint,
        &treasury_ata,
    )?;

    // Create payment terms - type-safe and validated!
    let (terms_pda, _sig) = client.create_payment_terms(
        TermsId::new("premium_monthly")?,    // Validated ID
        UsdcAmount::from_usdc(10.0),         // Clear amount
        PaymentPeriod::days(30)?,            // Readable period
        PaymentPeriod::days(3)?,             // Readable grace
        "Premium Monthly",
    )?;

    println!("Payee PDA: {}", payee_pda);
    println!("Terms PDA: {}", terms_pda);

    Ok(())
}
```

**Key Improvements:**
- ✅ Type-safe amounts and periods
- ✅ Validated terms ID
- ✅ Self-documenting code
- ✅ Compile-time validation

### Example 2: Keeper Service - Execute Payment

**Old Code:**
```rust
// keeper/src/executor.rs
use tally_sdk::{
    SimpleTallySigningClient, ExecutePaymentArgs,
    pda, ata,
};

pub async fn execute_payment_for_agreement(
    client: &SimpleTallySigningClient,
    agreement: &PaymentAgreement,
    terms: &PaymentTerms,
) -> Result<String> {
    let keeper = client.signer();

    // Manual PDA computation
    let payee_pda = pda::payee_address(&terms.payee)?;
    let config_pda = pda::config_address()?;
    let delegate_pda = pda::delegate_address()?;

    // Manual ATA computation
    let payer_ata = ata::get_associated_token_address_with_program(
        &agreement.payer,
        &terms.usdc_mint,
        TokenProgram::Token,
    )?;

    // Execute payment
    let args = ExecutePaymentArgs {
        expected_amount: terms.amount_usdc,
    };

    let sig = client.execute_payment(
        &keeper,
        &agreement_pda,
        &terms_pda,
        args,
    )?;

    Ok(sig)
}
```

**New Code:**
```rust
// keeper/src/executor.rs
use tally_sdk::{
    SimpleTallySigningClient, UsdcAmount,
    pda, ata,
};

pub async fn execute_payment_for_agreement(
    client: &SimpleTallySigningClient,
    agreement: &PaymentAgreement,
    terms: &PaymentTerms,
) -> Result<String> {
    let keeper = client.signer();

    // Simplified PDA computation
    let payee_pda = pda::payee(&terms.payee)?;
    let config_pda = pda::config()?;
    let delegate_pda = pda::delegate()?;

    // Same ATA computation
    let payer_ata = ata::get_associated_token_address_with_program(
        &agreement.payer,
        &terms.usdc_mint,
        TokenProgram::Token,
    )?;

    // Execute payment - type-safe amount
    let expected = UsdcAmount::from_microlamports(terms.amount_usdc);

    let sig = client.execute_payment(
        &keeper,
        &agreement_pda,
        &terms_pda,
        expected,  // Type-safe!
    )?;

    // Better logging
    println!("Payment executed: {} to {}", expected, payee_pda);

    Ok(sig)
}
```

**Key Improvements:**
- ✅ Simpler PDA API
- ✅ Type-safe amounts
- ✅ Better logging with Display impl

### Example 3: Dashboard Service - Calculate Analytics

**Old Code:**
```rust
// dashboard/src/analytics.rs
pub fn calculate_revenue_metrics(agreements: &[PaymentAgreement]) -> RevenueMetrics {
    let mut total_revenue: u64 = 0;
    let mut platform_fees: u64 = 0;

    for agreement in agreements {
        total_revenue += agreement.last_amount;

        // Manual fee calculation
        let fee_bps = 250_u16;  // 2.5%
        let fee = (agreement.last_amount as u128 * fee_bps as u128 / 10_000) as u64;
        platform_fees += fee;
    }

    RevenueMetrics {
        total_revenue,
        platform_fees,
        // How do we display these?
        total_revenue_usd: (total_revenue as f64 / 1_000_000.0),
        platform_fees_usd: (platform_fees as f64 / 1_000_000.0),
    }
}
```

**New Code:**
```rust
// dashboard/src/analytics.rs
use tally_sdk::{UsdcAmount, BasisPoints};

pub fn calculate_revenue_metrics(agreements: &[PaymentAgreement]) -> RevenueMetrics {
    let mut total_revenue = UsdcAmount::ZERO;
    let mut platform_fees = UsdcAmount::ZERO;

    let platform_fee_rate = BasisPoints::new(250).unwrap();  // 2.5%

    for agreement in agreements {
        let payment = UsdcAmount::from_microlamports(agreement.last_amount);
        total_revenue = total_revenue.saturating_add(payment);

        // Type-safe fee calculation
        let fee = platform_fee_rate.apply_to(payment);
        platform_fees = platform_fees.saturating_add(fee);
    }

    // Display formatting built-in!
    println!("Total Revenue: {}", total_revenue);  // "$1,234.56"
    println!("Platform Fees: {} ({})", platform_fees, platform_fee_rate);  // "$30.86 (2.50%)"

    RevenueMetrics {
        total_revenue: total_revenue.microlamports(),
        platform_fees: platform_fees.microlamports(),
        total_revenue_usd: total_revenue.usdc(),
        platform_fees_usd: platform_fees.usdc(),
    }
}
```

**Key Improvements:**
- ✅ No manual calculations
- ✅ Saturating arithmetic (no overflow panics)
- ✅ Built-in display formatting
- ✅ Type safety prevents mixing amounts with fees

---

## Common Migration Patterns

### Pattern 1: Converting Existing u64 Amounts

```rust
// You have existing u64 values from database or on-chain accounts
let raw_amount: u64 = agreement.last_amount;

// Wrap in UsdcAmount for type safety
let amount = UsdcAmount::from_microlamports(raw_amount);

// Now you can use type-safe operations
println!("Payment: {}", amount);  // Auto-formatting
let fee = platform_fee.apply_to(amount);  // Type-safe calculation
```

### Pattern 2: Accepting User Input

```rust
// CLI: Parse user input
use clap::Parser;

#[derive(Parser)]
struct CreateTermsArgs {
    #[arg(long)]
    amount: f64,  // User enters dollars

    #[arg(long)]
    period_days: u64,  // User enters days
}

// Convert to type-safe values
let amount = UsdcAmount::from_usdc(args.amount);
let period = PaymentPeriod::days(args.period_days)?;  // Validated!

// Use with SDK
client.create_payment_terms(terms_id, amount, period, grace, name)?;
```

### Pattern 3: Working with Fetched Accounts

```rust
// Fetch on-chain account
let terms = client.get_payment_terms(&terms_pda)?
    .ok_or("Terms not found")?;

// Convert to type-safe values
let amount = UsdcAmount::from_microlamports(terms.amount_usdc);
let period = PaymentPeriod::from_seconds(terms.period_secs)?;
let terms_id = TermsId::from_padded_bytes(&terms.terms_id)?;

// Now use with type safety
println!("Plan: {} - {} every {}", terms_id, amount, period);
// Output: "Plan: premium_monthly - $10.00 every 30 days"
```

### Pattern 4: Backward Compatibility

If you need to support both old and new code during migration:

```rust
// Create a compatibility layer
pub mod compat {
    use super::*;

    // Old function signature (deprecated)
    #[deprecated(since = "1.0.0", note = "Use create_terms_typed instead")]
    pub fn create_terms_raw(
        client: &SimpleTallySigningClient,
        terms_id: &str,
        amount_usdc: u64,
        period_secs: u64,
        grace_secs: u64,
        name: &str,
    ) -> Result<(Pubkey, String)> {
        // Convert to new types
        let terms_id = TermsId::new(terms_id)?;
        let amount = UsdcAmount::from_microlamports(amount_usdc);
        let period = PaymentPeriod::from_seconds(period_secs)?;
        let grace = PaymentPeriod::from_seconds(grace_secs)?;

        // Call new implementation
        create_terms_typed(client, terms_id, amount, period, grace, name)
    }

    // New function signature
    pub fn create_terms_typed(
        client: &SimpleTallySigningClient,
        terms_id: TermsId,
        amount: UsdcAmount,
        period: PaymentPeriod,
        grace: PaymentPeriod,
        name: &str,
    ) -> Result<(Pubkey, String)> {
        client.create_payment_terms(terms_id, amount, period, grace, name)
    }
}
```

---

## Testing Your Migration

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tally_sdk::{UsdcAmount, BasisPoints, PaymentPeriod, TermsId};

    #[test]
    fn test_amount_conversion() {
        // Old way
        let old_amount: u64 = 10_000_000;

        // New way
        let new_amount = UsdcAmount::from_usdc(10.0);

        // Should be equivalent
        assert_eq!(new_amount.microlamports(), old_amount);
    }

    #[test]
    fn test_fee_calculation() {
        let amount = UsdcAmount::from_usdc(100.0);
        let fee_rate = BasisPoints::new(250).unwrap();  // 2.5%
        let fee = fee_rate.apply_to(amount);

        assert_eq!(fee.usdc(), 2.5);
    }

    #[test]
    fn test_terms_id_validation() {
        // Valid IDs
        assert!(TermsId::new("premium_monthly").is_ok());
        assert!(TermsId::new("basic-plan").is_ok());

        // Invalid IDs
        assert!(TermsId::new("").is_err());
        assert!(TermsId::new("premium monthly").is_err());  // spaces
        assert!(TermsId::new("a".repeat(33)).is_err());     // too long
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_create_terms_with_new_types() {
        let client = test_client();

        // Old way - raw values
        let old_result = client.create_payment_terms_raw(
            "test_plan",
            10_000_000,
            2_592_000,
            259_200,
            "Test Plan"
        );

        // New way - type-safe
        let new_result = client.create_payment_terms(
            TermsId::new("test_plan")?,
            UsdcAmount::from_usdc(10.0),
            PaymentPeriod::days(30)?,
            PaymentPeriod::days(3)?,
            "Test Plan",
        );

        // Both should produce same PDAs
        assert_eq!(old_result?.0, new_result?.0);
    }
}
```

---

## Troubleshooting

### Compilation Errors

**Error:** `expected u64, found UsdcAmount`

```rust
// Before (won't compile)
let amount_usdc: u64 = UsdcAmount::from_usdc(10.0);

// After (correct)
let amount_usdc: u64 = UsdcAmount::from_usdc(10.0).microlamports();
// or
let amount = UsdcAmount::from_usdc(10.0);  // Keep as UsdcAmount
```

**Error:** `expected &[u8], found &TermsId`

```rust
// Before (won't compile)
let terms_pda = pda::payment_terms(&payee, &terms_id)?;

// After (correct)
let terms_pda = pda::payment_terms(&payee, &terms_id.to_padded_bytes())?;
// or better, use the new PDA API that accepts TermsId directly
let terms_pda = pda::payment_terms(&payee, &terms_id)?;
```

### Runtime Errors

**Error:** `Invalid terms ID: contains invalid characters`

```rust
// Cause: User input not validated
let terms_id = TermsId::new(user_input)?;  // Might fail!

// Solution: Validate and provide helpful error
let terms_id = TermsId::new(user_input).map_err(|e| {
    format!("Invalid terms ID: {}. Use only letters, numbers, underscores, and hyphens.", e)
})?;
```

**Error:** `Basis points 11000 exceeds maximum 10000`

```rust
// Cause: Invalid fee percentage
let fee = BasisPoints::new(11_000)?;  // Error: > 100%

// Solution: Validate user input
if user_fee_percent > 100.0 {
    return Err("Fee cannot exceed 100%".into());
}
let fee = BasisPoints::from_percentage(user_fee_percent)?;
```

---

## Checklist

Before deploying migrated code, verify:

### Code Changes
- [ ] All `u64` amounts replaced with `UsdcAmount`
- [ ] All `u16` fee values replaced with `BasisPoints`
- [ ] All period values replaced with `PaymentPeriod`
- [ ] All string terms IDs replaced with `TermsId`
- [ ] PDA function calls updated to new simplified API
- [ ] Error handling updated for new error variants

### Testing
- [ ] Unit tests pass with new types
- [ ] Integration tests pass
- [ ] Manual testing on devnet completed
- [ ] CLI commands tested with new types
- [ ] Keeper service tested with new types

### Documentation
- [ ] Update internal docs/wikis
- [ ] Update code comments
- [ ] Update CLI help text if needed

### Deployment
- [ ] Staged rollout plan
- [ ] Rollback plan if issues arise
- [ ] Monitoring for new error types

---

## Getting Help

### Resources

- **SDK Documentation:** `/home/rodzilla/projects/tally/tally-protocol/sdk/README.md`
- **Type Safety Docs:** `/home/rodzilla/projects/tally/tally-protocol/sdk/src/types.rs`
- **PDA Docs:** `/home/rodzilla/projects/tally/tally-protocol/sdk/src/pda.rs`
- **Examples:** `/home/rodzilla/projects/tally/tally-protocol/examples/`

### Common Questions

**Q: Do I need to update all code at once?**
A: No! You can migrate incrementally. Start with new code and update old code as you touch it.

**Q: Will old code break?**
A: Some old patterns are deprecated but still work. However, primitive types won't automatically convert to newtypes - you'll get compile errors that are easy to fix.

**Q: Can I mix old and new styles?**
A: Yes, during migration. Use `.microlamports()`, `.raw()`, etc. to convert between newtype and primitive values.

**Q: What if I find a bug?**
A: File an issue or reach out to the SDK maintainers. Include code samples and error messages.

---

## Summary

The Tally SDK improvements provide:

1. **Type Safety** - Prevent unit confusion and invalid values
2. **Better DX** - Self-documenting, easier to use
3. **Fewer Errors** - Validation at construction, not runtime
4. **Simpler API** - 75% reduction in PDA functions
5. **Better Errors** - Specific, actionable error messages

The migration effort is worthwhile - your code will be safer, clearer, and easier to maintain.

**Estimated Migration Time per Component:**
- Small CLI (< 500 LOC): 1-2 hours
- Medium service (< 2000 LOC): 4-6 hours
- Large keeper service (> 5000 LOC): 1-2 days

Most of the work is mechanical find-and-replace with some validation added.

---

**Last Updated:** November 10, 2025
**SDK Version:** 1.0.0 (pre-release improvements)
