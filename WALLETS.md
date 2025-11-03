# Development Wallets

This document describes the development wallets used for testing the Tally CLI.

## Wallet Locations

All wallets are stored in `~/.config/solana/` following Solana CLI conventions.

### Platform Authority Wallet

**Location:** `~/.config/solana/id.json`
**Purpose:** Platform authority for global configuration and admin operations
**Network:** Localnet (default Solana CLI wallet)

This is the main wallet used by the Solana CLI and should be used for:
- Initializing the global program configuration
- Platform-level administrative operations
- Funding other development wallets

### Merchant Dev Wallet

**Location:** `~/.config/solana/merchant-dev.json`
**Pubkey:** `9zV3QZuoPDU6Lo7z8sH9b5k6mSWWN2uy5Gdh7GcFoAY4`
**Purpose:** Testing merchant operations
**Network:** Localnet
**Balance:** 10 SOL (funded from platform authority)

This wallet is used for:
- Testing merchant initialization
- Creating subscription plans
- Testing the interactive init wizard
- Merchant-specific operations

## Usage

### Using Platform Authority Wallet (Default)

The platform authority wallet is used by default:
```bash
solana balance
# 500000000 SOL
```

### Using Merchant Dev Wallet

Specify the merchant wallet explicitly:
```bash
# Check balance
solana balance -k ~/.config/solana/merchant-dev.json
# 10 SOL

# Use with Tally CLI
tally-merchant init --authority ~/.config/solana/merchant-dev.json
```

## Funding Wallets (Localnet)

To fund additional wallets on localnet:

```bash
# From platform authority (default wallet)
solana transfer <recipient-pubkey> 10 --allow-unfunded-recipient
```

## Recreating Wallets

### Platform Authority
```bash
# Backup existing wallet first!
cp ~/.config/solana/id.json ~/.config/solana/id.json.backup

# Generate new wallet (will be funded by validator reset)
solana-keygen new --no-bip39-passphrase --outfile ~/.config/solana/id.json
```

### Merchant Dev Wallet
```bash
# Generate new wallet
solana-keygen new --no-bip39-passphrase --outfile ~/.config/solana/merchant-dev.json

# Fund with 10 SOL from platform authority
solana transfer <new-pubkey> 10 --allow-unfunded-recipient
```

## Security Notes

⚠️ **These are development wallets only!**
- Never use these wallets on mainnet
- Never send real funds to these addresses
- These wallets are for local testing only
- Seed phrases are stored in plaintext for development convenience

## Related Files

- CLI configuration: `~/.config/tally/config.toml`
- Platform env: `../tally-platform/.env.local`
- Validator data: `../tally-platform/test-ledger/`
