# Tally Merchant CLI

Command-line tool for managing recurring USDC subscriptions on Solana.

## What is Tally Merchant CLI?

Tally Merchant CLI lets you create and manage subscription plans without writing code or building payment infrastructure. If you're a creator, SaaS builder, or community operator on Solana, this tool helps you:

- **Set up recurring payments** in minutes, not weeks
- **Create subscription plans** with custom pricing and billing periods
- **Monitor your revenue** with real-time subscription tracking
- **Manage subscribers** without manual payment collection

Built on the [Tally Protocol](https://github.com/Tally-Pay/tally-protocol), which enables recurring USDC payments through secure, user-controlled delegate approvals on Solana.

## Installation

```bash
cargo install --git https://github.com/Tally-Pay/tally-cli
```

Or build from source:

```bash
git clone https://github.com/Tally-Pay/tally-cli
cd tally-cli
cargo build --release
```

## Quick Start

### 1. Register as a Merchant

```bash
tally-merchant init-merchant \
  --treasury <YOUR_USDC_TOKEN_ACCOUNT> \
  --fee-bps 50
```

### 2. Create a Subscription Plan

```bash
tally-merchant create-plan \
  --merchant <MERCHANT_ADDRESS> \
  --id "premium" \
  --name "Premium Plan" \
  --price 10000000 \
  --period 2592000 \
  --grace 86400
```

### 3. View Your Plans

```bash
tally-merchant list-plans --merchant <MERCHANT_ADDRESS>
```

### 4. Monitor Subscriptions

```bash
tally-merchant list-subs --plan <PLAN_ADDRESS>
```

## Command Reference

### Global Options

All commands support these global flags:

- `--rpc-url <URL>` - Override default RPC endpoint
- `--output <FORMAT>` - Output format: `human` (default) or `json`
- `--program-id <PUBKEY>` - Override Tally program ID
- `--usdc-mint <PUBKEY>` - Override USDC mint address

### Merchant Operations

#### Initialize Merchant

Register as a merchant to start accepting subscriptions.

```bash
tally-merchant init-merchant \
  --treasury <USDC_TOKEN_ACCOUNT> \
  --fee-bps <FEE_BASIS_POINTS>
```

**Arguments:**
- `--treasury` - Your USDC token account for receiving payments
- `--fee-bps` - Merchant fee in basis points (e.g., 50 = 0.5%)
- `--authority` (optional) - Custom authority keypair path

**Example:**
```bash
tally-merchant init-merchant \
  --treasury EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v \
  --fee-bps 100
```

#### Show Merchant Details

Display merchant account information.

```bash
tally-merchant show-merchant --merchant <MERCHANT_PDA>
```

**Output includes:**
- Merchant authority
- Treasury account
- Fee rate
- Total plans created

**Example with JSON output:**
```bash
tally-merchant show-merchant \
  --merchant HkDq...xyz \
  --output json
```

### Plan Management

#### Create Plan

Create a new subscription plan with custom pricing.

```bash
tally-merchant create-plan \
  --merchant <MERCHANT_PDA> \
  --id "<PLAN_ID>" \
  --name "<PLAN_NAME>" \
  --price <USDC_MICRO_UNITS> \
  --period <SECONDS> \
  --grace <SECONDS>
```

**Arguments:**
- `--merchant` - Merchant PDA address
- `--id` - Unique plan identifier (e.g., "premium")
- `--name` - Display name for the plan
- `--price` - Price in USDC micro-units (1 USDC = 1,000,000)
- `--period` - Billing period in seconds
- `--grace` - Grace period before cancellation (seconds)

**Common Billing Periods:**
- 1 day: 86400
- 1 week: 604800
- 1 month (30 days): 2592000
- 1 year (365 days): 31536000

**Example:**
```bash
# $10/month plan with 1-day grace period
tally-merchant create-plan \
  --merchant HkDq...xyz \
  --id "basic" \
  --name "Basic Plan" \
  --price 10000000 \
  --period 2592000 \
  --grace 86400
```

#### Update Plan Terms

Modify pricing, billing period, or grace period for existing plans.

```bash
tally-merchant update-plan-terms \
  --plan <PLAN_PDA> \
  [--price <NEW_PRICE>] \
  [--period <NEW_PERIOD>] \
  [--grace-period <NEW_GRACE>]
```

**Notes:**
- At least one field must be provided
- Changes apply to new subscriptions immediately
- Existing subscriptions continue with original terms

**Example:**
```bash
# Update price to $15/month
tally-merchant update-plan-terms \
  --plan 8rPq...abc \
  --price 15000000
```

#### List Plans

View all subscription plans for a merchant.

```bash
tally-merchant list-plans --merchant <MERCHANT_PDA>
```

**Example with JSON:**
```bash
tally-merchant list-plans \
  --merchant HkDq...xyz \
  --output json
```

#### Deactivate Plan

Stop accepting new subscriptions to a plan (existing subscriptions continue).

```bash
tally-merchant deactivate-plan --plan <PLAN_PDA>
```

**Note:** This command is currently blocked pending Anchor program implementation.

### Subscription Management

#### Start Subscription

Create a new subscription to a plan (testing/demo purposes).

```bash
tally-merchant start-subscription \
  --plan <PLAN_PDA> \
  --subscriber <SUBSCRIBER_PUBKEY> \
  [--allowance-periods <MULTIPLIER>] \
  [--trial-duration-secs <TRIAL_SECONDS>]
```

**Arguments:**
- `--plan` - Plan PDA to subscribe to
- `--subscriber` - Subscriber's public key
- `--allowance-periods` - Allowance multiplier (default: 3)
- `--trial-duration-secs` - Trial period: 604800 (7 days), 1209600 (14 days), or 2592000 (30 days)

**Example:**
```bash
tally-merchant start-subscription \
  --plan 8rPq...abc \
  --subscriber 5jKm...xyz \
  --allowance-periods 3 \
  --trial-duration-secs 604800
```

#### Cancel Subscription

Cancel an active subscription and revoke token delegate.

```bash
tally-merchant cancel-subscription \
  --subscription <SUBSCRIPTION_PDA> \
  --subscriber <SUBSCRIBER_PUBKEY>
```

#### Renew Subscription

Process a subscription renewal (keeper operation).

```bash
tally-merchant renew-subscription \
  --subscription <SUBSCRIPTION_PDA>
```

**Note:** Requires sufficient USDC allowance on subscriber's token account.

#### Close Subscription

Reclaim rent from a canceled subscription (~0.00099792 SOL).

```bash
tally-merchant close-subscription \
  --subscription <SUBSCRIPTION_PDA> \
  --subscriber <SUBSCRIBER_PUBKEY>
```

**Requirements:**
- Subscription must be in Canceled status
- Only the subscriber can close their subscription

#### List Subscriptions

View all subscriptions for a plan.

```bash
tally-merchant list-subs --plan <PLAN_PDA>
```

**Example with JSON:**
```bash
tally-merchant list-subs \
  --plan 8rPq...abc \
  --output json
```

#### Show Subscription Details

Display detailed subscription information.

```bash
tally-merchant show-subscription --subscription <SUBSCRIPTION_PDA>
```

**Output includes:**
- Plan and subscriber addresses
- Subscription status
- Next renewal date
- Total renewals processed
- Payment history

### Analytics & Monitoring

#### Dashboard Overview

View merchant statistics and revenue metrics.

```bash
tally-merchant dashboard overview --merchant <MERCHANT_PDA>
```

**Output includes:**
- Total active subscriptions
- Revenue metrics
- Subscription events
- Growth trends

#### Plan Analytics

View analytics for a specific plan.

```bash
tally-merchant dashboard analytics --plan <PLAN_PDA>
```

#### Event Monitoring

Monitor real-time subscription events.

```bash
tally-merchant dashboard events \
  --merchant <MERCHANT_PDA> \
  [--since <UNIX_TIMESTAMP>]
```

#### Subscription List

Enhanced subscription listing with merchant-wide view.

```bash
tally-merchant dashboard subscriptions \
  --merchant <MERCHANT_PDA> \
  [--active-only]
```

### Inspection Commands

#### Show Configuration

Display global Tally protocol configuration.

```bash
tally-merchant show-config
```

**Output includes:**
- Program ID
- Platform authority
- Maximum platform fee
- Keeper fee rate

### Testing & Development

#### Simulate Events

Generate synthetic subscription events for testing monitoring systems.

```bash
tally-merchant simulate-events \
  --merchant <MERCHANT_PDA> \
  [--plan <PLAN_PDA>] \
  [--scenario <SCENARIO>] \
  [--rate <EVENTS_PER_MIN>] \
  [--duration <SECONDS>]
```

**Scenarios:**
- `normal` - Typical subscription activity
- `high-churn` - High cancellation rate
- `growth` - Rapid subscriber growth

**Output Options:**
- `stdout` - Print to terminal (default)
- `websocket` - Send to WebSocket endpoint
- `file` - Write to JSON file

**Example:**
```bash
# Simulate 60 events/min for 5 minutes
tally-merchant simulate-events \
  --merchant HkDq...xyz \
  --scenario growth \
  --rate 60 \
  --duration 300 \
  --output stdout
```

## JSON Output

All commands support `--output json` for machine-readable output:

```bash
# Get merchant info as JSON
tally-merchant show-merchant \
  --merchant HkDq...xyz \
  --output json | jq .

# List plans as JSON and pipe to another tool
tally-merchant list-plans \
  --merchant HkDq...xyz \
  --output json | jq '.[] | select(.active == true)'
```

## Environment Variables

Configure defaults via environment variables (optional):

```bash
export TALLY_RPC_URL="https://api.devnet.solana.com"
export TALLY_DEFAULT_OUTPUT_FORMAT="json"
export USDC_DECIMALS_DIVISOR="1000000"
```

## How Tally Works

**Recurring payments without repeated signatures.** Subscribers approve a bounded USDC allowance (default 3× the subscription price) through a token delegate. The Tally program automatically collects payments each billing period while subscribers maintain full control—they can revoke access anytime.

**Key features:**
- Bounded security (3× price allowance, not unlimited approvals)
- User control (cancel and revoke anytime)
- Automated renewals (no manual payment collection)
- Transparent on-chain state (verifiable subscription status)

## Who This Is For

**Creators** - Launch membership subscriptions without payment infrastructure
**SaaS Builders** - Add recurring billing to your Solana dApp
**Community Operators** - Automate Discord/Telegram access based on subscription status

## Common Workflows

### Setting Up Your First Plan

```bash
# 1. Initialize merchant
tally-merchant init-merchant \
  --treasury YOUR_USDC_ACCOUNT \
  --fee-bps 50

# 2. Create a plan
tally-merchant create-plan \
  --merchant MERCHANT_ADDRESS \
  --id "monthly" \
  --name "Monthly Membership" \
  --price 10000000 \
  --period 2592000 \
  --grace 86400

# 3. View your plans
tally-merchant list-plans --merchant MERCHANT_ADDRESS
```

### Monitoring Active Subscriptions

```bash
# View all subscriptions for a plan
tally-merchant list-subs --plan PLAN_ADDRESS

# Check subscription details
tally-merchant show-subscription --subscription SUB_ADDRESS

# View merchant dashboard
tally-merchant dashboard overview --merchant MERCHANT_ADDRESS
```

### Testing Subscription Flow

```bash
# 1. Start a subscription
tally-merchant start-subscription \
  --plan PLAN_ADDRESS \
  --subscriber SUBSCRIBER_PUBKEY

# 2. Simulate a renewal
tally-merchant renew-subscription \
  --subscription SUB_ADDRESS

# 3. Cancel subscription
tally-merchant cancel-subscription \
  --subscription SUB_ADDRESS \
  --subscriber SUBSCRIBER_PUBKEY

# 4. Close and reclaim rent
tally-merchant close-subscription \
  --subscription SUB_ADDRESS \
  --subscriber SUBSCRIBER_PUBKEY
```

## Troubleshooting

### Common Issues

**"Merchant account does not exist"**
- Verify you've run `init-merchant` first
- Check that the merchant PDA address is correct

**"Insufficient SOL for transaction"**
- Ensure your wallet has enough SOL for transaction fees
- Minimum 0.005 SOL recommended

**"Invalid USDC token account"**
- Verify your treasury is a valid USDC Associated Token Account
- Ensure you're using the correct USDC mint for your network

**"Subscription already exists"**
- Each subscriber can only have one subscription per plan
- Cancel existing subscription before creating a new one

### Getting Help

- Check [Tally Protocol Documentation](https://github.com/Tally-Pay/tally-protocol)
- Review [CLAUDE.md](./CLAUDE.md) for development guidance
- Open an issue on [GitHub](https://github.com/Tally-Pay/tally-cli/issues)

## Learn More

- [Tally Protocol](https://github.com/Tally-Pay/tally-protocol) - Smart contract and SDK documentation
- [Subscription Lifecycle](https://github.com/Tally-Pay/tally-protocol/blob/main/docs/SUBSCRIPTION_LIFECYCLE.md) - How subscriptions work
- [Security Model](https://github.com/Tally-Pay/tally-protocol/blob/main/docs/MULTI_MERCHANT_LIMITATION.md) - Delegate approvals and limitations

> **Note:** Platform administration commands (init-config, withdraw-platform-fees, transfer-authority) are available in the separate [tally-admin-cli](https://github.com/Tally-Pay/tally-admin-cli) tool.

## License

MIT License - see LICENSE file for details
