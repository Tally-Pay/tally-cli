# Tally Merchant CLI

**Production command-line tool for managing recurring USDC subscriptions on Solana.**

## What is Tally Merchant CLI?

Tally Merchant CLI is a production-ready tool for merchants to manage subscription plans and monitor revenue. Built on the [Tally Protocol](https://github.com/Tally-Pay/tally-protocol), which enables recurring USDC payments through secure, user-controlled delegate approvals.

### Key Features

- **Create & manage subscription plans** with custom pricing and billing periods
- **Monitor revenue & subscribers** with real-time analytics
- **View subscription details** and payment history
- **JSON output** for integration with other tools

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

All commands support these flags:

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

Display merchant account information including authority, treasury, and fee rate.

```bash
tally-merchant show-merchant --merchant <MERCHANT_PDA>
```

**Example with JSON:**
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

### Subscription Monitoring

#### List Subscriptions

View all subscriptions for a specific plan.

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

Display detailed information about a specific subscription.

```bash
tally-merchant show-subscription --subscription <SUBSCRIPTION_PDA>
```

**Output includes:**
- Plan and subscriber addresses
- Subscription status (Active/Inactive)
- Next renewal date
- Total renewals processed
- Payment history

### Analytics & Dashboard

#### Dashboard Overview

View merchant statistics and revenue metrics.

```bash
tally-merchant dashboard overview --merchant <MERCHANT_PDA>
```

**Output includes:**
- Total active subscriptions
- Revenue metrics
- Recent subscription events
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

### Global Configuration

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

## JSON Output

All commands support `--output json` for machine-readable output:

```bash
# Get merchant info as JSON
tally-merchant show-merchant \
  --merchant HkDq...xyz \
  --output json | jq .

# List plans as JSON and filter active plans
tally-merchant list-plans \
  --merchant HkDq...xyz \
  --output json | jq '.[] | select(.active == true)'

# Get subscription count
tally-merchant list-subs \
  --plan 8rPq...abc \
  --output json | jq 'length'
```

## Environment Variables

Configure defaults via environment variables:

```bash
export TALLY_RPC_URL="https://api.devnet.solana.com"
export TALLY_DEFAULT_OUTPUT_FORMAT="json"
export USDC_DECIMALS_DIVISOR="1000000"
```

## How Subscriptions Work

### For Merchants

1. **Create Plans**: Define pricing, billing period, and grace period
2. **Share Blinks**: Give subscribers your Solana Action/Blink URL
3. **Monitor Revenue**: Use dashboard commands to track subscriptions
4. **Update Plans**: Adjust pricing or terms as needed

### For Subscribers (Not in This CLI)

Subscribers interact with your subscription plans through:
- **Solana Actions/Blinks**: Shared links that open in wallets
- **Your dApp Frontend**: Custom UI using Tally SDK
- **Mobile Wallets**: Phantom, Solflare, etc.

They sign transactions with their own wallet—you never have their private keys.

### Payment Flow

1. **Subscribe**: User approves USDC delegate via Blink/dApp
2. **First Payment**: Immediate charge upon subscription
3. **Renewals**: Keeper bots process renewals automatically
4. **Cancel**: User can revoke delegate anytime in their wallet

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

# Check specific subscription details
tally-merchant show-subscription --subscription SUB_ADDRESS

# View merchant dashboard
tally-merchant dashboard overview --merchant MERCHANT_ADDRESS
```

### Updating Pricing

```bash
# Update plan price
tally-merchant update-plan-terms \
  --plan PLAN_ADDRESS \
  --price 15000000

# Update billing period
tally-merchant update-plan-terms \
  --plan PLAN_ADDRESS \
  --period 5184000

# Update multiple fields
tally-merchant update-plan-terms \
  --plan PLAN_ADDRESS \
  --price 12000000 \
  --grace-period 604800
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

**"Plan not found"**
- Verify the plan PDA address is correct
- Check that the plan exists using `list-plans`

### Getting Help

- Check [Tally Protocol Documentation](https://github.com/Tally-Pay/tally-protocol)
- Review [CLAUDE.md](./CLAUDE.md) for development guidance
- Open an issue on [GitHub](https://github.com/Tally-Pay/tally-cli/issues)

## Security

- **Merchant Keys**: Store securely, they control your revenue
- **Treasury Account**: Use a dedicated USDC account
- **Never Share**: Private keys, keypair files, or seed phrases
- **Read-Only Operations**: Most commands only read blockchain state

## Learn More

- [Tally Protocol](https://github.com/Tally-Pay/tally-protocol) - Smart contract and SDK documentation
- [Subscription Lifecycle](https://github.com/Tally-Pay/tally-protocol/blob/main/docs/SUBSCRIPTION_LIFECYCLE.md) - How subscriptions work end-to-end
- [Security Model](https://github.com/Tally-Pay/tally-protocol/blob/main/docs/MULTI_MERCHANT_LIMITATION.md) - Delegate approvals and limitations

> **Note**: Platform administration commands (init-config, withdraw-platform-fees, transfer-authority) are in the separate [tally-admin-cli](https://github.com/Tally-Pay/tally-admin-cli) tool.

## License

MIT License - see LICENSE file for details
