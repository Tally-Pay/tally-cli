# Tally CLI

Command-line tool for managing recurring USDC subscriptions on Solana.

## What is Tally CLI?

Tally CLI lets you create and manage subscription plans without writing code or building payment infrastructure. If you're a creator, SaaS builder, or community operator on Solana, this tool helps you:

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
tally-cli init-merchant \
  --treasury <YOUR_USDC_TOKEN_ACCOUNT> \
  --fee-bps 50
```

### 2. Create a Subscription Plan

```bash
tally-cli create-plan \
  --merchant <MERCHANT_ADDRESS> \
  --id "premium" \
  --name "Premium Plan" \
  --price 10000000 \
  --period 2592000 \
  --grace 86400
```

### 3. View Your Plans

```bash
tally-cli list-plans --merchant <MERCHANT_ADDRESS>
```

### 4. Monitor Subscriptions

```bash
tally-cli list-subs --plan <PLAN_ADDRESS>
```

## Key Commands

- `init-merchant` - Register as a merchant to accept subscriptions
- `create-plan` - Create a new subscription plan with pricing
- `list-plans` - View all plans for a merchant
- `list-subs` - View all subscriptions for a plan
- `deactivate-plan` - Stop accepting new subscriptions to a plan
- `dashboard` - View analytics and subscription metrics

Run `tally-cli --help` for full command documentation.

> **Note:** Platform administration commands (init-config, withdraw-fees, pause/unpause) are available in the separate [tally-admin-cli](https://github.com/Tally-Pay/tally-admin-cli) tool.

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

## Learn More

- [Tally Protocol](https://github.com/Tally-Pay/tally-protocol) - Smart contract and SDK documentation
- [Subscription Lifecycle](https://github.com/Tally-Pay/tally-protocol/blob/main/docs/SUBSCRIPTION_LIFECYCLE.md) - How subscriptions work
- [Security Model](https://github.com/Tally-Pay/tally-protocol/blob/main/docs/MULTI_MERCHANT_LIMITATION.md) - Delegate approvals and limitations

## License

MIT License - see LICENSE file for details
