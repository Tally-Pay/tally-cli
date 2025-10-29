# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Tally CLI** is a command-line interface for the Tally Protocol - a Solana-native subscription platform enabling recurring USDC payments through SPL Token delegate approvals. The CLI provides merchant, plan, and subscription management capabilities built on top of the `tally-sdk` Rust SDK.

### Product Context

Tally Protocol enables:
- **Merchants**: Create subscription plans with flexible pricing and billing periods
- **Subscribers**: Approve bounded USDC allowances (3× price default) for automatic renewals
- **Keepers**: Execute permissionless renewal payments via delegate transfers
- **Platform**: Earn tiered fees while providing infrastructure

Key architectural concept: **Single-delegate architecture** - subscribers approve a merchant-specific delegate PDA for automatic payment collection. This enables seamless recurring billing without repeated user signatures.

### Core Value Propositions
- **Blink-native distribution**: Shareable subscription links via Solana Actions
- **Bounded delegate security**: 3× price allowance with user revocation
- **Standards-based**: Built on Solana Actions and SPL Token standards
- **Zero custom frontend**: Subscription deployment via link sharing

## Architecture

### Module Organization

```
src/
├── main.rs           # CLI entry point, command routing
├── lib.rs            # Library exports for testing
├── config.rs         # Configuration management (env vars, defaults)
├── commands/         # Command implementations
│   ├── mod.rs        # Command exports
│   ├── init_config.rs      # Initialize global program config
│   ├── init_merchant.rs    # Register new merchant
│   ├── create_plan.rs      # Create subscription plan
│   ├── list_plans.rs       # List merchant plans
│   ├── list_subs.rs        # List plan subscriptions
│   ├── deactivate_plan.rs  # Deactivate plan
│   ├── withdraw_fees.rs    # Admin fee withdrawal
│   ├── dashboard.rs        # Analytics and monitoring
│   └── simulate_events/    # Event simulation for testing
│       ├── mod.rs
│       └── tests.rs
└── utils/            # Shared utilities
    ├── mod.rs
    └── formatting.rs # Display formatting helpers
```

### Key Dependencies

- **tally-sdk**: Core SDK for Tally Protocol interaction (from main tally-protocol repo)
- **anchor-client/anchor-lang**: Solana program interaction
- **clap**: CLI argument parsing with derive macros
- **tokio**: Async runtime
- **tracing**: Logging infrastructure

### SDK Integration

The CLI exclusively communicates with the on-chain Solana program through the `tally-sdk` layer. Direct program calls are forbidden - all operations must use SDK methods:

```rust
// Correct: SDK method
tally_client.create_plan(&merchant, &args).await?;

// Forbidden: Direct program call
program.request().instruction(/* ... */).send()?;
```

This architecture ensures:
- Consistent error handling and validation
- Type safety through SDK abstractions
- Easy testing via SDK mocking
- Clear separation of concerns

### Configuration System

`TallyCliConfig` centralizes configuration with environment variable support:

```rust
// Environment Variables
TALLY_RPC_URL                    # Default: https://api.devnet.solana.com
TALLY_DEFAULT_OUTPUT_FORMAT      # Default: human
USDC_DECIMALS_DIVISOR            # Default: 1_000_000
BASIS_POINTS_DIVISOR             # Default: 100.0
TALLY_DEFAULT_EVENTS_LOOKBACK_SECS  # Default: 3600

// Usage
let config = TallyCliConfig::new();
let usdc_display = config.format_usdc(micro_units);
let fee_pct = config.format_fee_percentage(fee_bps);
```

## Development Commands

### Building

```bash
# Development build
cargo build

# Production build with optimizations
cargo build --release

# Check compilation without building
cargo check
```

### Testing

```bash
# Run all tests with nextest (preferred)
cargo nextest run

# Run specific test
cargo nextest run test_name

# Run tests with standard cargo
cargo test

# Run single test with standard cargo
cargo test test_name
```

The project uses **cargo nextest** as the primary test runner for faster execution and better output formatting. Tests are located in:
- Unit tests: `#[cfg(test)]` modules within source files
- Currently in: `config.rs`, `simulate_events/mod.rs`

### Code Quality

```bash
# Run clippy lints (must pass with zero warnings)
cargo clippy

# Run clippy with all warnings as errors
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

**IMPORTANT**: All code must:
- Pass `cargo clippy` with **zero warnings**
- Never suppress lints with `#[allow]` directives - fix underlying issues
- Use `#![forbid(unsafe_code)]` at crate level (already enforced)
- Follow DRY principles with pure, testable functions
- NEVER mention Claude in commit history, commit messages, commit authors or any comments or documentation in the project.
- NEVER include CLAUDE.md in any commit while also never adding it to gitignore.

### Running the CLI

```bash
# Display help
cargo run -- --help

# Run command with development build
cargo run -- init-merchant --treasury <PUBKEY> --fee-bps 50

# Run with custom RPC URL
cargo run -- --rpc-url https://api.devnet.solana.com list-plans --merchant <PUBKEY>

# Run with JSON output
cargo run -- --output json list-plans --merchant <PUBKEY>

# Using release build
cargo run --release -- <command>

# Direct binary execution after build
./target/release/tally-cli <command>
```

### Command Examples

```bash
# Initialize global config (one-time setup)
tally-cli init-config \
  --platform-authority <PUBKEY> \
  --max-platform-fee-bps 1000

# Initialize merchant
tally-cli init-merchant \
  --treasury <USDC_ATA_PUBKEY> \
  --fee-bps 150

# Create subscription plan
tally-cli create-plan \
  --merchant <MERCHANT_PDA> \
  --id "premium" \
  --name "Premium Plan" \
  --price 10000000 \
  --period 2592000 \
  --grace 86400

# List merchant plans
tally-cli list-plans --merchant <MERCHANT_PDA>

# List plan subscriptions
tally-cli list-subs --plan <PLAN_PDA>
```

## Development Guidelines

### SDK-First Development

All new CLI features MUST:
1. Use SDK methods exclusively for program interaction
2. Never make direct program calls
3. Implement proper error handling from SDK responses
4. Follow existing command patterns in `commands/` directory

### Command Implementation Pattern

```rust
// 1. Define request struct
pub struct CreateXRequest<'a> {
    pub field: &'a str,
    // ...
}

// 2. Implement execute function
pub async fn execute(
    tally_client: &SimpleTallyClient,
    request: &CreateXRequest<'_>,
    config: &TallyCliConfig,
) -> Result<String> {
    // Parse and validate inputs
    let pubkey = Pubkey::from_str(request.field)?;

    // Call SDK method (NEVER direct program calls)
    let result = tally_client.some_method(&pubkey).await?;

    // Format and return output
    Ok(format!("Success: {}", result))
}

// 3. Wire up in main.rs Commands enum and execute_command
```

### Testing Requirements

New features must include:
- Unit tests for logic and validation
- Test coverage for error cases
- SDK mock usage for integration-style tests
- All tests must pass via `cargo nextest run`

### Code Style

- **Idiomatic Rust**: Leverage std traits, use iterator methods
- **Safety**: No unsafe code, checked arithmetic
- **Modularity**: Keep modules focused and reasonably sized
- **Documentation**: Doc comments on public items
- **Error Context**: Use `anyhow::Context` to add error context

## Important Protocol Concepts

### Account Structure

- **Config (138 bytes)**: Global program config at PDA `["config", program_id]`
- **Merchant (108 bytes)**: Merchant config at PDA `["merchant", authority, program_id]`
- **Plan (129 bytes)**: Subscription plan at PDA `["plan", merchant, plan_id, program_id]`
- **Subscription (120 bytes)**: User subscription at PDA `["subscription", plan, subscriber, program_id]`

### Payment Flow

1. **Start**: Subscriber approves delegate → first payment → subscription active
2. **Renewal**: Keeper executes via delegate → fees distributed → next renewal scheduled
3. **Cancel**: Subscriber revokes delegate → subscription inactive
4. **Close**: Reclaim rent from canceled subscription (~0.00099792 SOL)

### Fee Distribution (per renewal)

1. Keeper: 0.5% (configurable, max 1%)
2. Platform: 1-2% (tier-based: Free=2%, Pro=1.5%, Enterprise=1%)
3. Merchant: Remainder (98-99%)

### Critical Limitations

**Single-Delegate Constraint**: SPL Token accounts support only ONE delegate. Subscribing to multiple merchants with the same USDC account overwrites previous delegates, breaking existing subscriptions.

**Mitigation**: CLI should eventually detect and warn about existing delegates before subscription operations.

## Related Projects

- **tally-protocol** (../tally-protocol): Main protocol repository containing:
  - Anchor program implementation
  - Rust SDK (`tally-sdk`)
  - TypeScript SDK
  - Comprehensive documentation

- **tally-branding** (../tally-branding): Brand strategy and messaging
  - Positioning: Blink-native subscription infrastructure
  - Target audiences: Creators, SaaS builders, community operators
  - Value props: Infrastructure simplification, viral distribution, bounded security
