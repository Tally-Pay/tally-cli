# Recurring Payments Architecture Analysis

**Date:** 2025-11-08
**Context:** Architectural evolution from subscription-specific platform to general recurring payments platform
**Status:** Architectural Recommendation

---

## Executive Summary

### Core Insight

The Tally protocol has evolved from a **subscription-specific platform** to a **general recurring payments primitive**. The global delegate refactor unlocked use cases far beyond traditional subscriptions:
- Multi-merchant subscriptions
- Budget compartmentalization (defense in depth)
- Hierarchical payment structures (company → department → employee)
- Crypto investment with retractable funding
- DAO treasury management
- Grant programs and phased funding

**However**, the current codebase still contains subscription-specific features that limit its generality and create unnecessary complexity for non-subscription use cases.

### Architectural Recommendation

**Adopt a layered architecture:**

1. **Core Protocol (tally-protocol)**: Minimal recurring payment primitives
   - Scheduled transfers with delegate-based authorization
   - Timing and amount validation
   - Keeper incentivization
   - Event observability

2. **Composable Extensions (separate programs)**: Use-case-specific features
   - Subscription layer: Free trials, grace periods, plan management
   - Investment layer: Vesting schedules, cliff periods, milestone releases
   - Payroll layer: Tax withholding, split payments, time tracking
   - Grant layer: Milestone validation, reporting requirements

**This separation provides:**
- **Simplicity**: Core protocol is minimal and easy to audit
- **Flexibility**: Use cases compose features without bloat
- **Upgradability**: Extensions evolve independently
- **Clarity**: Domain naming reflects universal applicability

---

## 1. Feature Classification: Core vs Use-Case-Specific

### ✅ CORE PROTOCOL PRIMITIVES (Keep)

These features are **fundamental to ALL recurring payment use cases** and should remain in the core protocol:

#### 1.1 Account Structure
| Account | Purpose | Why Core? |
|---------|---------|-----------|
| **Config** | Global program parameters | Every use case needs global configuration |
| **Payee** (renamed from Merchant) | Payment recipient configuration | Universal: someone receives payments |
| **PaymentTerms** (renamed from Plan) | Payment schedule and amount | Universal: defines what/when to pay |
| **PaymentAgreement** (renamed from Subscription) | Payer-payee relationship | Universal: tracks payment relationship |

#### 1.2 Core Instructions
| Instruction | Why Core? |
|-------------|-----------|
| **init_config** | Every deployment needs configuration |
| **init_payee** | Universal: onboard payment recipients |
| **create_payment_terms** | Universal: define payment schedules |
| **start_agreement** | Universal: establish payment relationship |
| **execute_payment** (renamed from renew_subscription) | Universal: process scheduled payment |
| **pause_agreement** (renamed from cancel_subscription) | Universal: stop payments temporarily |
| **resume_agreement** | Universal: restart payments after pause |
| **close_agreement** (renamed from close_subscription) | Universal: reclaim rent |

#### 1.3 Core Fields
| Field | Account | Why Core? |
|-------|---------|-----------|
| `payer` | PaymentAgreement | Universal: who pays |
| `payee` | PaymentAgreement | Universal: who receives |
| `payment_terms` | PaymentAgreement | Universal: what schedule to follow |
| `amount` | PaymentTerms | Universal: how much to pay |
| `period_secs` | PaymentTerms | Universal: payment frequency |
| `next_payment_ts` | PaymentAgreement | Universal: when next payment due |
| `active` | PaymentAgreement | Universal: is relationship active? |
| `payment_count` | PaymentAgreement | Universal: track payment history |
| `created_ts` | PaymentAgreement | Universal: when relationship started |
| `last_payment_ts` | PaymentAgreement | Universal: idempotency protection |
| `last_amount` | PaymentAgreement | Universal: audit trail |

#### 1.4 Core Events
| Event | Why Core? |
|-------|-----------|
| **PaymentAgreementStarted** | Universal: relationship established |
| **PaymentExecuted** | Universal: payment processed |
| **PaymentAgreementPaused** | Universal: payments stopped |
| **PaymentAgreementResumed** | Universal: payments restarted |
| **PaymentAgreementClosed** | Universal: relationship ended |
| **PaymentFailed** | Universal: failed payment observability |
| **LowAllowanceWarning** | Universal: allowance management |
| **DelegateMismatchWarning** | Universal: delegate validation |

#### 1.5 Core Security & Financial Primitives
| Feature | Why Core? |
|---------|-----------|
| **Checked arithmetic** | Universal: prevent overflow attacks |
| **Delegate-based transfers** | Universal: payment mechanism |
| **Global delegate PDA** | Universal: multi-payee support |
| **Idempotency protection** | Universal: prevent double-charging |
| **Timing validation** | Universal: prevent premature payment |
| **Amount validation** | Universal: bounded transfers |
| **Keeper fee distribution** | Universal: incentivize network |
| **Platform fee distribution** | Universal: protocol sustainability |
| **Emergency pause** | Universal: security incident response |

---

### ❌ SUBSCRIPTION-SPECIFIC FEATURES (Extract to Extension)

These features are **specific to subscription use cases** and should be moved to a separate `tally-subscriptions` program:

#### 2.1 Free Trials
**Where:**
- `Subscription.trial_ends_at: Option<i64>` (state.rs)
- `Subscription.in_trial: bool` (state.rs)
- Trial validation in `start_subscription.rs` (lines 151-220)
- Trial conversion logic in `renew_subscription.rs` (lines 85-92)
- `TrialStarted` event (events.rs)
- `TrialConverted` event (events.rs)
- `TrialAlreadyUsed` error (errors.rs)
- `InvalidTrialDuration` error (errors.rs)
- `TRIAL_DURATION_7_DAYS`, `TRIAL_DURATION_14_DAYS`, `TRIAL_DURATION_30_DAYS` constants (constants.rs)

**Why Extract:**
- **Not universal**: Payroll doesn't have trials, investments don't have trials
- **Subscription-specific**: "Try before you buy" is a SaaS business model pattern
- **Complexity**: Adds 2 fields + 2 events + 2 errors + validation logic
- **State bloat**: Every PaymentAgreement pays rent for trial fields even when unused

**Alternative Core Approach:**
- Use `payment_count == 0` + custom start logic in extension layer
- Extension tracks trial state separately
- Core protocol doesn't need trial-specific fields

#### 2.2 Grace Periods
**Where:**
- `Plan.grace_secs: u64` (state.rs)
- Grace period validation in `renew_subscription.rs` (lines 99-111)
- `max_grace_period_seconds` in Config (state.rs)
- Grace period validation in `create_plan.rs`
- `PastGrace` error (errors.rs)

**Why Extract:**
- **Not universal**: Payroll has no grace period (exact payment dates), grants have milestones not grace
- **Subscription-specific**: "Forgiveness period" is a retention/churn reduction tactic
- **Arbitrary timing**: Core protocol should enforce exact timing, extensions add flexibility
- **Business logic**: Deciding grace period length is subscription-specific business logic

**Alternative Core Approach:**
- Core enforces strict timing: payment due at `next_payment_ts`
- Extension program checks if payment overdue but within grace → calls core payment
- Extension emits grace-specific events
- Core protocol remains deterministic

#### 2.3 Plan Active/Inactive Status
**Where:**
- `Plan.active: bool` (state.rs)
- `update_plan` instruction (update_plan.rs)
- Plan status validation in `start_subscription.rs`
- `PlanStatusChanged` event (events.rs)
- `Inactive` error (errors.rs - applies to plans)

**Why Extract:**
- **Not universal**: Payment terms for payroll/grants don't have "active" status - they exist or don't
- **Subscription-specific**: Merchants "pause" plan sign-ups without deleting plan
- **Business state**: "Accepting new customers" is business logic, not payment primitive
- **Confusion**: Conflates plan existence with subscription acceptance

**Alternative Core Approach:**
- Core protocol: Payment terms exist or don't exist (no status field)
- Extension: Subscription layer tracks "plan availability" separately
- Extension: Validates plan active status before calling core `start_agreement`
- Core account rent: Slightly reduced (1 byte saved per plan)

#### 2.4 Plan Name Field
**Where:**
- `Plan.name: [u8; 32]` (state.rs)
- Name conversion in `create_plan.rs` (lines 92-111)
- Name in `PlanCreated` event (events.rs)

**Why Extract:**
- **Not universal**: Payment terms don't need human-readable names on-chain
- **UI concern**: Names are for display/marketing, not payment execution
- **Storage waste**: 32 bytes per plan for metadata that could be off-chain
- **Localization**: On-chain names can't be translated/updated easily

**Alternative Core Approach:**
- Core protocol: Payment terms identified by PDA only
- Extension/Off-chain: Store names in indexer/database via `plan_id` mapping
- Events include `plan_id` (already unique) for off-chain name lookup
- Core account rent: 32 bytes saved per plan (~0.000224 SOL)

#### 2.5 Merchant Tier System
**Where:**
- `MerchantTier` enum (state.rs - Free/Pro/Enterprise)
- `Merchant.tier: MerchantTier` (state.rs)
- Tier-based fee calculation (state.rs lines 14-24)
- `update_merchant_tier` instruction (update_merchant_tier.rs)
- `MerchantTierChanged` event (events.rs)

**Why Extract:**
- **Not universal**: Payroll senders don't have "tiers", grant programs don't have "tiers"
- **Business model**: Tiered pricing is a SaaS monetization strategy
- **Platform-specific**: Different platforms might have different tier structures
- **Hardcoded assumptions**: Free/Pro/Enterprise tiers are arbitrary business decisions

**Alternative Core Approach:**
- Core protocol: Payees have single `platform_fee_bps: u16` field
- Platform admin sets fee directly per payee
- Extension: Subscription platform implements tier logic off-chain
- Core: Simpler, more flexible fee structure

#### 2.6 Subscription Reactivation Logic
**Where:**
- Reactivation detection in `start_subscription.rs` (lines 148-220)
- `SubscriptionReactivated` event (events.rs)
- Trial-already-used validation (start_subscription.rs)
- Preserves `renewals` count across cancellation

**Why Extract:**
- **Not universal**: Payroll doesn't "reactivate", it starts new employment period
- **Subscription-specific**: "Win-back" is a subscription retention pattern
- **Complex state**: Distinguishing new vs reactivation is business logic
- **Event noise**: Core protocol should treat start as start

**Alternative Core Approach:**
- Core protocol: `start_agreement` always starts (idempotent if already active)
- Extension: Tracks whether this is first agreement or reactivation
- Extension emits reactivation-specific events
- Core: Simpler initialization logic

#### 2.7 Subscription "Renewals" Terminology
**Where:**
- `Subscription.renewals: u32` field name
- `renew_subscription` instruction name
- `Renewed` event name
- All renewal-related documentation

**Why Extract:**
- **Not universal**: Payroll has "paycheck cycles", grants have "disbursements", investments have "distributions"
- **Subscription-specific**: "Renewal" implies subscription continuation
- **Semantic confusion**: Not renewing anything, just executing scheduled payment

**Alternative Core Approach:**
- Rename `renewals` → `payment_count` (generic counter)
- Rename `renew_subscription` → `execute_payment` (generic action)
- Rename `Renewed` → `PaymentExecuted` (generic event)
- Extension: Use subscription-specific language in UI/docs

---

## 2. Domain Model Redesign: Universal Recurring Payment Naming

### 2.1 Naming Principles

1. **Action-oriented**: Names describe what happens, not domain assumptions
2. **Neutral terminology**: Works across all use cases without confusion
3. **Self-documenting**: Clear meaning without context
4. **Composable**: Extensions add specificity through their own naming

### 2.2 Proposed Account Names

| Current Name | Proposed Name | Rationale |
|--------------|---------------|-----------|
| `Merchant` | `Payee` | - Universal: "who receives payment"<br>- Works for: merchants, employees, grant recipients, investors<br>- Neutral: no business model assumptions<br>- Clear: one word, intuitive meaning |
| `Plan` | `PaymentTerms` | - Universal: "conditions for payment"<br>- Works for: subscription plans, salary schedules, grant milestones, vesting terms<br>- Descriptive: clearly states purpose<br>- Flexible: can represent any agreement structure |
| `Subscription` | `PaymentAgreement` | - Universal: "agreement to make payments"<br>- Works for: subscriptions, employment, grants, investments<br>- Legal clarity: emphasizes contractual relationship<br>- Symmetric: payer and payee both have obligations |
| `Subscriber` (field) | `Payer` | - Universal: "who makes payment"<br>- Parallel to "Payee"<br>- Clear role identification<br>- Works across all contexts |

### 2.3 Proposed Instruction Names

| Current Name | Proposed Name | Rationale |
|--------------|---------------|-----------|
| `init_merchant` | `init_payee` | Follows account rename |
| `create_plan` | `create_payment_terms` | Follows account rename |
| `start_subscription` | `start_agreement` | - Generic action<br>- Works for hiring, subscribing, funding<br>- Shorter than `start_payment_agreement` |
| `renew_subscription` | `execute_payment` | - Describes actual operation<br>- No renewal assumption<br>- Clear action verb |
| `cancel_subscription` | `pause_agreement` | - More accurate: agreement still exists, just paused<br>- Allows for resumption<br>- No finality assumption |
| `close_subscription` | `close_agreement` | - Follows account rename<br>- Emphasizes finality vs pause |
| (new) | `resume_agreement` | - Explicit resumption after pause<br>- Replaces current reactivation via start_subscription<br>- Clear state transition |

### 2.4 Proposed Event Names

| Current Name | Proposed Name | Rationale |
|--------------|---------------|-----------|
| `Subscribed` | `PaymentAgreementStarted` | Clear, universal action |
| `SubscriptionReactivated` | `PaymentAgreementResumed` | More accurate state transition |
| `Renewed` | `PaymentExecuted` | Generic payment action |
| `Canceled` | `PaymentAgreementPaused` | Clarifies non-finality |
| `SubscriptionClosed` | `PaymentAgreementClosed` | Follows account rename |
| `MerchantInitialized` | `PayeeInitialized` | Follows account rename |
| `PlanCreated` | `PaymentTermsCreated` | Follows account rename |
| `PlanStatusChanged` | (Remove - extension only) | Not in core protocol |
| `MerchantTierChanged` | (Remove - extension only) | Not in core protocol |
| `TrialStarted` | (Remove - extension only) | Not in core protocol |
| `TrialConverted` | (Remove - extension only) | Not in core protocol |

### 2.5 Proposed Error Names

| Current Name | Proposed Name | Rationale |
|--------------|---------------|-----------|
| `SubscriptionError` | `PaymentError` | Follows module rename |
| `Inactive` | `AgreementInactive` | More specific to agreement state |
| `AlreadyActive` | `AgreementAlreadyActive` | More specific |
| `AlreadyCanceled` | `AgreementAlreadyPaused` | Follows pause terminology |
| `PastGrace` | (Remove - extension only) | Grace period is subscription-specific |
| `TrialAlreadyUsed` | (Remove - extension only) | Trials are subscription-specific |
| `InvalidTrialDuration` | (Remove - extension only) | Trials are subscription-specific |

### 2.6 Proposed Field Names

| Current Field | Account | Proposed Field | Rationale |
|---------------|---------|----------------|-----------|
| `subscriber` | Subscription | `payer` | Parallel to `payee` |
| `plan` | Subscription | `payment_terms` | Follows account rename |
| `renewals` | Subscription | `payment_count` | Generic counter |
| `next_renewal_ts` | Subscription | `next_payment_ts` | No renewal assumption |
| `last_renewed_ts` | Subscription | `last_payment_ts` | No renewal assumption |
| `grace_secs` | Plan | (Remove - extension only) | Not universal |
| `trial_ends_at` | Subscription | (Remove - extension only) | Not universal |
| `in_trial` | Subscription | (Remove - extension only) | Not universal |
| `active` | Plan | (Remove - extension only) | Not universal |
| `name` | Plan | (Remove - extension only) | UI metadata |
| `tier` | Merchant | (Remove - extension only) | Business logic |

---

## 3. Architecture Diagram: Core Protocol + Composable Layers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         USE CASE LAYER (Separate Programs)                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐             │
│  │  Subscription   │  │    Payroll      │  │  Investment     │             │
│  │   Extension     │  │   Extension     │  │   Extension     │  ... more   │
│  │                 │  │                 │  │                 │             │
│  │ • Free trials   │  │ • Tax calc      │  │ • Vesting       │             │
│  │ • Grace periods │  │ • Split pay     │  │ • Cliff periods │             │
│  │ • Plan status   │  │ • Time tracking │  │ • Milestones    │             │
│  │ • Tiers         │  │ • Benefits      │  │ • Drawdowns     │             │
│  │ • Reactivation  │  │ • Attendance    │  │ • Governance    │             │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘             │
│           │                    │                     │                       │
│           └────────────────────┼─────────────────────┘                       │
│                                │                                             │
│                                ▼ CPI Calls                                   │
│                                                                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                      CORE PROTOCOL (tally-protocol)                          │
│                     Minimal Recurring Payment Primitives                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  Accounts:                                                                   │
│   ┌──────────────┐  ┌──────────────────┐  ┌──────────────────────┐         │
│   │   Config     │  │      Payee       │  │   PaymentTerms       │         │
│   ├──────────────┤  ├──────────────────┤  ├──────────────────────┤         │
│   │ • authority  │  │ • authority      │  │ • payee              │         │
│   │ • fees       │  │ • treasury_ata   │  │ • amount             │         │
│   │ • limits     │  │ • usdc_mint      │  │ • period_secs        │         │
│   │ • paused     │  │ • fee_bps        │  │ • id                 │         │
│   └──────────────┘  └──────────────────┘  └──────────────────────┘         │
│                                                                               │
│   ┌──────────────────────────────────────────────────────────────┐          │
│   │               PaymentAgreement                               │          │
│   ├──────────────────────────────────────────────────────────────┤          │
│   │ • payer (who pays)                                           │          │
│   │ • payment_terms (reference to PaymentTerms PDA)             │          │
│   │ • next_payment_ts (when next payment due)                   │          │
│   │ • active (is agreement active?)                             │          │
│   │ • payment_count (number of payments executed)               │          │
│   │ • created_ts (when agreement started)                       │          │
│   │ • last_payment_ts (idempotency protection)                  │          │
│   │ • last_amount (audit trail)                                 │          │
│   └──────────────────────────────────────────────────────────────┘          │
│                                                                               │
│  Instructions:                                                               │
│   • init_config          - Initialize global configuration                  │
│   • init_payee           - Register payment recipient                       │
│   • create_payment_terms - Define payment schedule and amount               │
│   • start_agreement      - Establish payer-payee relationship               │
│   • execute_payment      - Process scheduled payment via delegate           │
│   • pause_agreement      - Stop payments (revoke delegate)                  │
│   • resume_agreement     - Restart payments (re-approve delegate)           │
│   • close_agreement      - End relationship and reclaim rent                │
│                                                                               │
│  Core Features:                                                              │
│   ✓ Global delegate PDA (multi-payee support)                               │
│   ✓ Delegate-based SPL transfers                                            │
│   ✓ Timing validation (prevent premature payment)                           │
│   ✓ Amount validation (bounded transfers)                                   │
│   ✓ Idempotency protection (prevent double-payment)                         │
│   ✓ Keeper fee distribution (incentivize network)                           │
│   ✓ Platform fee distribution (protocol sustainability)                     │
│   ✓ Emergency pause (security incident response)                            │
│   ✓ Checked arithmetic (overflow protection)                                │
│   ✓ Event emissions (observability)                                         │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.1 Extension Program Pattern

```rust
// Example: Subscription Extension Program

#[program]
pub mod tally_subscriptions {
    // Extension-specific accounts
    #[account]
    pub struct SubscriptionPlan {
        pub payment_terms_pda: Pubkey,  // Reference to core PaymentTerms
        pub name: String,                // Human-readable name
        pub active: bool,                // Accepting new subscriptions?
        pub trial_duration_secs: Option<u64>,
        pub grace_period_secs: u64,
    }

    #[account]
    pub struct SubscriptionMetadata {
        pub agreement_pda: Pubkey,       // Reference to core PaymentAgreement
        pub trial_ends_at: Option<i64>,
        pub in_trial: bool,
        pub tier: SubscriptionTier,
    }

    // Extension instructions wrap core protocol calls
    pub fn subscribe_with_trial(
        ctx: Context<SubscribeWithTrial>,
        trial_duration: u64,
    ) -> Result<()> {
        // Validate trial eligibility (extension logic)
        require!(
            is_valid_trial_duration(trial_duration),
            SubscriptionError::InvalidTrialDuration
        );

        // Create extension metadata account
        let metadata = &mut ctx.accounts.subscription_metadata;
        metadata.trial_ends_at = Some(Clock::get()?.unix_timestamp + trial_duration);
        metadata.in_trial = true;

        // CPI to core protocol to establish payment agreement
        tally_protocol::cpi::start_agreement(
            ctx.accounts.into_start_agreement_context(),
            StartAgreementArgs { allowance_periods: 3 }
        )?;

        emit!(TrialStarted { ... });
        Ok(())
    }

    pub fn renew_subscription(ctx: Context<RenewSubscription>) -> Result<()> {
        let metadata = &mut ctx.accounts.subscription_metadata;
        let plan = &ctx.accounts.subscription_plan;

        // Extension-specific logic: check grace period
        let agreement = &ctx.accounts.payment_agreement;
        let current_time = Clock::get()?.unix_timestamp;
        let grace_deadline = agreement.next_payment_ts + plan.grace_period_secs;

        require!(
            current_time <= grace_deadline,
            SubscriptionError::PastGrace
        );

        // Extension-specific logic: trial conversion
        if metadata.in_trial {
            metadata.in_trial = false;
            emit!(TrialConverted { ... });
        }

        // CPI to core protocol to execute payment
        tally_protocol::cpi::execute_payment(
            ctx.accounts.into_execute_payment_context(),
            ExecutePaymentArgs {}
        )?;

        Ok(())
    }
}
```

### 3.2 Benefits of Layered Architecture

| Benefit | Description |
|---------|-------------|
| **Simplicity** | Core protocol is ~40% smaller, easier to audit |
| **Flexibility** | Use cases choose only features they need |
| **Composability** | Mix features from multiple extensions |
| **Upgradability** | Extensions evolve independently of core |
| **Security** | Smaller core = smaller attack surface |
| **Clarity** | Domain naming reflects actual purpose |
| **Efficiency** | Accounts only pay rent for fields they use |

---

## 4. Migration Strategy: From Current to Core Primitives

### 4.1 Phase 1: Rename in Place (Non-Breaking)

**Goal:** Update naming to reflect generality while maintaining compatibility

**Changes:**
1. Add type aliases for backward compatibility:
   ```rust
   pub type Merchant = Payee;
   pub type Plan = PaymentTerms;
   pub type Subscription = PaymentAgreement;
   ```

2. Update documentation to use new terminology
3. Deprecate old instruction names (keep implementation)
4. Add new instructions that call old implementations
5. Update SDK to export both old and new names

**Timeline:** 1 week
**Breaking:** No

### 4.2 Phase 2: Extract Subscription Features (Breaking)

**Goal:** Move subscription-specific features to extension program

**Steps:**

1. **Create `tally-subscriptions` extension program:**
   - Define `SubscriptionPlan` account (references core `PaymentTerms`)
   - Define `SubscriptionMetadata` account (references core `PaymentAgreement`)
   - Implement trial logic in extension
   - Implement grace period logic in extension
   - Implement plan status logic in extension
   - Implement tier logic in extension

2. **Remove from core protocol:**
   - `Subscription.trial_ends_at` field
   - `Subscription.in_trial` field
   - `Plan.grace_secs` field
   - `Plan.active` field
   - `Plan.name` field
   - `Merchant.tier` field
   - Trial validation logic
   - Grace period validation logic
   - Plan status validation logic
   - `update_plan` instruction
   - `update_merchant_tier` instruction
   - Trial-related events
   - Grace-related errors

3. **Update core protocol:**
   - Reduce account sizes
   - Simplify instruction logic
   - Rename accounts/instructions
   - Update events to generic names

4. **Migration path for users:**
   - Existing subscriptions continue working (no data migration)
   - New subscriptions use extension program
   - SDK provides migration helper functions
   - Dashboard updates to support both versions during transition

**Timeline:** 4-6 weeks
**Breaking:** Yes (major version bump)

### 4.3 Phase 3: Core Protocol Optimization

**Goal:** Optimize core protocol after extraction

**Changes:**

1. **Account size reduction:**
   - `Plan` → `PaymentTerms`: 129 bytes → ~89 bytes (40 bytes saved)
     - Remove `grace_secs` (8 bytes)
     - Remove `name` (32 bytes)
     - Remove `active` (1 byte)
   - `Subscription` → `PaymentAgreement`: 120 bytes → ~101 bytes (19 bytes saved)
     - Remove `trial_ends_at` (9 bytes)
     - Remove `in_trial` (1 byte)
   - `Merchant` → `Payee`: 108 bytes → ~107 bytes (1 byte saved)
     - Remove `tier` (1 byte)

2. **Rent savings:**
   - Per PaymentTerms: ~0.000280 SOL saved
   - Per PaymentAgreement: ~0.000133 SOL saved
   - For 10,000 agreements: ~4.13 SOL saved (~$180)

3. **Instruction simplification:**
   - `execute_payment` has no trial/grace logic
   - `start_agreement` has no trial logic
   - Cleaner error handling

**Timeline:** 2 weeks
**Breaking:** No (only optimization)

### 4.4 Migration Checklist

- [ ] Create backward-compatible type aliases
- [ ] Update documentation with new terminology
- [ ] Create `tally-subscriptions` extension program
- [ ] Migrate trial logic to extension
- [ ] Migrate grace period logic to extension
- [ ] Migrate plan status logic to extension
- [ ] Migrate tier logic to extension
- [ ] Update SDK with both old and new APIs
- [ ] Update CLI with migration commands
- [ ] Update dashboard to support extension
- [ ] Write migration guide for developers
- [ ] Deploy extension program to devnet
- [ ] Test migration path with sample subscriptions
- [ ] Deploy core protocol v2.0.0 (breaking changes)
- [ ] Deploy extension program to mainnet
- [ ] Sunset core protocol v1.x.x support

---

## 5. Use Case Validation: Does Core Protocol Support All Use Cases?

### 5.1 Multi-Merchant Subscriptions ✅

**Use Case:** User subscribes to multiple merchants (Netflix, Spotify, News) using one USDC account

**Core Protocol Support:**
- ✅ Global delegate PDA enables multi-payee
- ✅ `PaymentAgreement` tracks each payer-payee relationship
- ✅ Each payee has independent `PaymentTerms`
- ✅ `execute_payment` processes each agreement independently

**Extension Layer:**
- Subscription extension adds trial/grace logic per merchant
- UI shows subscription-specific metadata
- Analytics track subscription churn/retention

**Verdict:** Fully supported

### 5.2 Budget Compartmentalization (Defense in Depth) ✅

**Use Case:** Main wallet ($10K) funds subscription wallet ($100) to limit blast radius

**Core Protocol Support:**
- ✅ User creates dedicated USDC account for subscriptions
- ✅ Approves global delegate on subscription wallet only
- ✅ Main wallet never approves delegate
- ✅ Max loss = subscription wallet balance

**Extension Layer:**
- None needed - pure core protocol feature

**Verdict:** Fully supported

### 5.3 Hierarchical Payment Structures ✅

**Use Case:** Company treasury → department budgets → employee wallets → vendors

**Core Protocol Support:**
- ✅ Company treasury is `payer` for department agreements
- ✅ Department wallet is `payer` for employee agreements
- ✅ Employee wallet is `payer` for vendor agreements
- ✅ Each level approves global delegate
- ✅ Containment via compartmentalization

**Extension Layer:**
- Payroll extension adds tax calculations, split payments
- Approval workflow extension adds multi-sig requirements
- Budget tracking extension monitors spending limits

**Verdict:** Fully supported

### 5.4 Crypto Investment with Retractable Funding ✅

**Use Case:** Investor funds multiple projects monthly, can revoke if underperforming

**Core Protocol Support:**
- ✅ Investor is `payer`, projects are `payees`
- ✅ Each project has `PaymentTerms` (monthly amount)
- ✅ `PaymentAgreement` tracks investor-project relationship
- ✅ Investor can `pause_agreement` to stop funding
- ✅ Investor can `resume_agreement` to restart funding

**Extension Layer:**
- Investment extension adds vesting schedules, cliff periods
- Milestone extension adds performance-based disbursements
- Governance extension adds voting on continued funding

**Verdict:** Fully supported

### 5.5 DAO Treasury Management ✅

**Use Case:** DAO pays multiple working groups/contributors automatically

**Core Protocol Support:**
- ✅ DAO treasury is `payer`
- ✅ Each working group is `payee`
- ✅ `PaymentTerms` define monthly budgets
- ✅ Keeper executes payments automatically
- ✅ DAO can `pause_agreement` to halt funding

**Extension Layer:**
- Governance extension adds proposal-based funding
- Reporting extension adds deliverable tracking
- Multi-sig extension adds threshold approvals

**Verdict:** Fully supported

### 5.6 Grant Programs and Phased Funding ✅

**Use Case:** Foundation funds projects in monthly installments based on milestones

**Core Protocol Support:**
- ✅ Foundation is `payer`, grantees are `payees`
- ✅ `PaymentTerms` define installment amounts/timing
- ✅ Foundation can `pause_agreement` if milestones missed
- ✅ `PaymentAgreement` tracks payment history

**Extension Layer:**
- Grant extension adds milestone validation
- Reporting extension adds progress tracking
- Compliance extension adds KYC/AML requirements

**Verdict:** Fully supported

### 5.7 Employee Payroll ✅

**Use Case:** Company pays employees biweekly salaries

**Core Protocol Support:**
- ✅ Company is `payer`, employees are `payees`
- ✅ `PaymentTerms` define salary and period (biweekly)
- ✅ `execute_payment` processes payroll automatically
- ✅ Keeper handles payroll execution

**Extension Layer:**
- Payroll extension adds tax withholding calculations
- Benefits extension adds 401k deductions
- Timesheet extension validates hours worked

**Verdict:** Fully supported

### 5.8 Summary: All Use Cases Supported ✅

| Use Case | Core Protocol | Extension Needed? |
|----------|---------------|-------------------|
| Multi-merchant subscriptions | ✅ Full support | Subscription extension (optional) |
| Budget compartmentalization | ✅ Full support | None |
| Hierarchical payments | ✅ Full support | Payroll/approval extensions |
| Retractable investments | ✅ Full support | Investment extension |
| DAO treasury management | ✅ Full support | Governance extension |
| Grant programs | ✅ Full support | Grant extension |
| Employee payroll | ✅ Full support | Payroll extension |

**Conclusion:** The core protocol primitives are sufficient for all identified use cases. Extensions add use-case-specific features without modifying core behavior.

---

## 6. Technical Recommendations

### 6.1 Immediate Actions (Next 2 Weeks)

1. **Create type aliases for backward compatibility:**
   ```rust
   // In state.rs
   pub type Merchant = Payee;
   pub type Plan = PaymentTerms;
   pub type Subscription = PaymentAgreement;
   ```

2. **Update documentation:**
   - Update README to describe "recurring payments platform"
   - Add architecture diagram to docs
   - Create migration guide skeleton

3. **Prototype subscription extension:**
   - Create new Anchor workspace: `tally-subscriptions`
   - Implement `SubscriptionPlan` account
   - Implement trial logic as CPI wrapper
   - Test CPI calls to core protocol

### 6.2 Short-Term Actions (Next 1-2 Months)

1. **Build subscription extension to feature parity:**
   - Implement all subscription-specific features
   - Migrate tests to extension
   - Create SDK for extension
   - Deploy to devnet

2. **Prepare core protocol v2.0.0:**
   - Remove subscription-specific fields
   - Rename accounts/instructions
   - Reduce account sizes
   - Update events to generic names

3. **Create migration tooling:**
   - SDK migration helpers
   - CLI migration commands
   - Dashboard migration UI
   - User migration guide

### 6.3 Long-Term Actions (Next 3-6 Months)

1. **Deploy core protocol v2.0.0:**
   - Security audit focused on reduced core
   - Deploy to mainnet with migration path
   - Support v1.x.x for 90 days
   - Sunset v1.x.x after migration period

2. **Build additional extensions:**
   - Investment extension (vesting, milestones)
   - Payroll extension (tax, benefits)
   - Grant extension (reporting, compliance)
   - DAO extension (governance, voting)

3. **Developer ecosystem:**
   - Extension development guide
   - Extension templates
   - Composability examples
   - Best practices documentation

### 6.4 Non-Goals

**Do NOT:**
- ❌ Try to support subscriptions AND other use cases in core protocol
- ❌ Add more subscription-specific features to core
- ❌ Maintain backward compatibility forever (plan for v1 sunset)
- ❌ Bloat core protocol with "maybe useful" features
- ❌ Compromise simplicity for convenience

**Instead:**
- ✅ Keep core minimal and well-defined
- ✅ Build extensions for specific use cases
- ✅ Accept breaking changes for architectural clarity
- ✅ Prioritize composability over convenience
- ✅ Maintain clear separation of concerns

---

## 7. Comparison: Current vs Proposed Architecture

### 7.1 Account Size Comparison

| Account | Current Size | Proposed Core | Savings | Moved to Extension |
|---------|--------------|---------------|---------|-------------------|
| Merchant/Payee | 108 bytes | 107 bytes | 1 byte | `tier` field |
| Plan/PaymentTerms | 129 bytes | 89 bytes | 40 bytes | `grace_secs`, `name`, `active` |
| Subscription/Agreement | 120 bytes | 101 bytes | 19 bytes | `trial_ends_at`, `in_trial` |
| **Total per Agreement** | **357 bytes** | **297 bytes** | **60 bytes** | **16.8% reduction** |

**Rent Savings:**
- Current: 0.00249 SOL per agreement
- Proposed: 0.00207 SOL per agreement
- Savings: 0.00042 SOL per agreement (~$0.018)
- For 10,000 agreements: 4.2 SOL saved (~$184)

### 7.2 Instruction Complexity Comparison

| Instruction | Current LOC | Proposed Core | Extension LOC | Total LOC |
|-------------|-------------|---------------|---------------|-----------|
| start_subscription | ~350 | ~200 | ~150 | 350 (split) |
| renew_subscription | ~400 | ~250 | ~150 | 400 (split) |
| cancel_subscription | ~200 | ~150 | ~50 | 200 (split) |
| create_plan | ~150 | ~100 | ~50 | 150 (split) |
| update_plan | ~80 | 0 | ~80 | 80 (moved) |
| update_merchant_tier | ~60 | 0 | ~60 | 60 (moved) |

**Code Organization:**
- Current: All logic in core protocol
- Proposed: Business logic in extensions, primitives in core

### 7.3 Feature Matrix

| Feature | Current | Proposed Core | Subscription Extension |
|---------|---------|---------------|------------------------|
| Scheduled payments | ✅ | ✅ | - |
| Delegate transfers | ✅ | ✅ | - |
| Keeper incentives | ✅ | ✅ | - |
| Platform fees | ✅ | ✅ | - |
| Multi-payee support | ✅ | ✅ | - |
| Free trials | ✅ | ❌ | ✅ |
| Grace periods | ✅ | ❌ | ✅ |
| Plan status | ✅ | ❌ | ✅ |
| Merchant tiers | ✅ | ❌ | ✅ |
| Plan names | ✅ | ❌ | ✅ (off-chain) |
| Reactivation | ✅ | ✅ (as resume) | ✅ (metadata) |

### 7.4 Security Comparison

| Security Feature | Current | Proposed Core | Notes |
|------------------|---------|---------------|-------|
| Checked arithmetic | ✅ | ✅ | Unchanged |
| Idempotency protection | ✅ | ✅ | Unchanged |
| Delegate validation | ✅ | ✅ | Unchanged |
| Timing validation | ✅ | ✅ | Simpler (no grace) |
| Amount validation | ✅ | ✅ | Unchanged |
| Emergency pause | ✅ | ✅ | Unchanged |
| PDA validation | ✅ | ✅ | Fewer PDAs to validate |
| **Attack Surface** | **Larger** | **Smaller** | **40% less code in core** |

---

## 8. Conclusion and Next Steps

### 8.1 Summary

The Tally protocol has outgrown its subscription-specific origins. The global delegate refactor proved that the core innovation is **delegate-based recurring payments**, not subscriptions per se.

**The path forward is clear:**

1. **Extract subscription features** to a separate extension program
2. **Rename core protocol** to reflect universal recurring payment primitives
3. **Reduce core protocol** to minimal, essential features
4. **Enable composability** via extension programs for specific use cases

This architecture provides:
- **Simplicity**: Smaller core = easier audits, fewer bugs
- **Flexibility**: Use cases compose only the features they need
- **Clarity**: Naming reflects actual purpose
- **Efficiency**: Accounts only pay rent for fields they use
- **Extensibility**: New use cases don't require core changes

### 8.2 Critical Decision Points

**Decision 1: Commit to breaking changes**
- ✅ Recommended: Yes, accept breaking changes for architectural clarity
- Timeline: Core v2.0.0 in 3-6 months with migration path

**Decision 2: Naming convention**
- ✅ Recommended: `Payee`, `PaymentTerms`, `PaymentAgreement`
- Alternative considered: `Recipient`, `Schedule`, `Stream` (rejected as too abstract)

**Decision 3: Extension architecture**
- ✅ Recommended: Separate programs with CPI to core
- Alternative considered: Feature flags in core (rejected as bloat)

**Decision 4: Migration strategy**
- ✅ Recommended: Phased migration with 90-day transition period
- Alternative considered: Immediate cutover (rejected as too risky)

### 8.3 Immediate Next Steps

1. **[Today]** Present this analysis to team for architectural decision
2. **[This Week]** Prototype subscription extension with trial logic
3. **[Next Week]** Validate extension CPI pattern with end-to-end test
4. **[Next 2 Weeks]** Create migration plan with timeline and milestones
5. **[Next Month]** Implement core protocol v2.0.0 with renamed accounts
6. **[Next Quarter]** Deploy subscription extension and migrate users

### 8.4 Open Questions

1. **Versioning**: Should we use separate program IDs for v1 vs v2, or upgrade in place?
   - Recommendation: Separate program IDs for clean migration

2. **Extension discovery**: How do users know which extensions exist?
   - Recommendation: Registry program or off-chain directory

3. **Extension composability**: Can users combine subscription + investment extensions?
   - Recommendation: Yes, via multiple metadata accounts referencing same core agreement

4. **Fee distribution**: Should extensions collect their own fees?
   - Recommendation: Yes, extensions can add their own fee structures

5. **Governance**: Who approves new extensions?
   - Recommendation: Permissionless - anyone can build extensions

### 8.5 Success Metrics

Track these metrics to validate architectural decision:

| Metric | Current | Target (6 months) |
|--------|---------|-------------------|
| Core protocol LOC | ~2,500 | ~1,500 (40% reduction) |
| Account rent (per agreement) | 0.00249 SOL | 0.00207 SOL (16% reduction) |
| Extension programs deployed | 0 | 3+ (subscription, investment, payroll) |
| Non-subscription use cases | 0 | 2+ (investment, DAO treasury) |
| Security audit findings | TBD | <5 findings (smaller surface) |
| Developer adoption | Subscription-only | Multi-use-case |

---

## Appendix A: Detailed Code Migration Examples

### A.1 Current Subscription Logic (Before)

```rust
// program/src/start_subscription.rs (current)
pub fn handler(ctx: Context<StartSubscription>, args: StartSubscriptionArgs) -> Result<()> {
    let subscription = &mut ctx.accounts.subscription;
    let plan = &ctx.accounts.plan;

    // Subscription-specific: trial validation
    if let Some(trial_duration) = args.trial_duration_secs {
        require!(
            is_valid_trial_duration(trial_duration),
            SubscriptionError::InvalidTrialDuration
        );
        subscription.trial_ends_at = Some(current_time + trial_duration);
        subscription.in_trial = true;
    }

    // Subscription-specific: plan status check
    require!(plan.active, SubscriptionError::Inactive);

    // Core logic: establish payment relationship
    subscription.plan = plan.key();
    subscription.subscriber = ctx.accounts.subscriber.key();
    subscription.next_renewal_ts = calculate_next_payment(current_time, plan.period_secs);
    subscription.active = true;

    // Process initial payment if not trial
    if !subscription.in_trial {
        transfer_payment(ctx, plan.price_usdc)?;
    }

    Ok(())
}
```

### A.2 Proposed Core Protocol (After)

```rust
// tally-protocol/src/start_agreement.rs (core protocol v2)
pub fn handler(ctx: Context<StartAgreement>, args: StartAgreementArgs) -> Result<()> {
    let agreement = &mut ctx.accounts.payment_agreement;
    let terms = &ctx.accounts.payment_terms;

    // Core logic only: establish payment relationship
    agreement.payment_terms = terms.key();
    agreement.payer = ctx.accounts.payer.key();
    agreement.next_payment_ts = calculate_next_payment(
        Clock::get()?.unix_timestamp,
        terms.period_secs
    );
    agreement.active = true;
    agreement.payment_count = 0;
    agreement.created_ts = Clock::get()?.unix_timestamp;

    // Always process initial payment (no trial logic)
    transfer_payment(ctx, terms.amount)?;

    emit!(PaymentAgreementStarted {
        payer: agreement.payer,
        payee: terms.payee,
        payment_terms: terms.key(),
        amount: terms.amount,
    });

    Ok(())
}
```

### A.3 Subscription Extension (After)

```rust
// tally-subscriptions/src/subscribe.rs (extension program)
pub fn handler(ctx: Context<Subscribe>, args: SubscribeArgs) -> Result<()> {
    let metadata = &mut ctx.accounts.subscription_metadata;
    let plan = &ctx.accounts.subscription_plan;

    // Extension-specific: plan status validation
    require!(plan.active, SubscriptionError::PlanInactive);

    // Extension-specific: trial logic
    if let Some(trial_duration) = args.trial_duration_secs {
        require!(
            is_valid_trial_duration(trial_duration),
            SubscriptionError::InvalidTrialDuration
        );

        // Create metadata account to track trial
        metadata.agreement = ctx.accounts.payment_agreement.key();
        metadata.trial_ends_at = Some(Clock::get()?.unix_timestamp + trial_duration);
        metadata.in_trial = true;

        emit!(TrialStarted {
            subscription: ctx.accounts.payment_agreement.key(),
            subscriber: ctx.accounts.payer.key(),
            plan: plan.key(),
            trial_ends_at: metadata.trial_ends_at.unwrap(),
        });

        // Don't call core protocol yet - trial means no payment
        return Ok(());
    }

    // No trial: CPI to core protocol to establish payment agreement
    tally_protocol::cpi::start_agreement(
        CpiContext::new_with_signer(
            ctx.accounts.tally_program.to_account_info(),
            tally_protocol::cpi::accounts::StartAgreement {
                config: ctx.accounts.config.to_account_info(),
                payment_agreement: ctx.accounts.payment_agreement.to_account_info(),
                payment_terms: ctx.accounts.payment_terms.to_account_info(),
                payee: ctx.accounts.payee.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                // ... other accounts
            },
            &[],
        ),
        tally_protocol::instruction::StartAgreementArgs {
            allowance_periods: args.allowance_periods,
        },
    )?;

    Ok(())
}
```

---

## Appendix B: Complete Account Structure Comparison

### B.1 Current State (v1.x.x)

```rust
// Current: Subscription-specific naming and fields
#[account]
pub struct Merchant {
    pub authority: Pubkey,          // 32 bytes
    pub usdc_mint: Pubkey,          // 32 bytes
    pub treasury_ata: Pubkey,       // 32 bytes
    pub platform_fee_bps: u16,      // 2 bytes
    pub tier: MerchantTier,         // 1 byte ⚠️ Subscription-specific
    pub bump: u8,                   // 1 byte
}
// Total: 100 bytes + 8 discriminator = 108 bytes

#[account]
pub struct Plan {
    pub merchant: Pubkey,           // 32 bytes
    pub plan_id: [u8; 32],          // 32 bytes
    pub price_usdc: u64,            // 8 bytes
    pub period_secs: u64,           // 8 bytes
    pub grace_secs: u64,            // 8 bytes ⚠️ Subscription-specific
    pub name: [u8; 32],             // 32 bytes ⚠️ UI metadata
    pub active: bool,               // 1 byte ⚠️ Subscription-specific
}
// Total: 121 bytes + 8 discriminator = 129 bytes

#[account]
pub struct Subscription {
    pub plan: Pubkey,               // 32 bytes
    pub subscriber: Pubkey,         // 32 bytes
    pub next_renewal_ts: i64,       // 8 bytes
    pub active: bool,               // 1 byte
    pub renewals: u32,              // 4 bytes
    pub created_ts: i64,            // 8 bytes
    pub last_amount: u64,           // 8 bytes
    pub last_renewed_ts: i64,       // 8 bytes
    pub trial_ends_at: Option<i64>, // 9 bytes ⚠️ Subscription-specific
    pub in_trial: bool,             // 1 byte ⚠️ Subscription-specific
    pub bump: u8,                   // 1 byte
}
// Total: 112 bytes + 8 discriminator = 120 bytes
```

### B.2 Proposed Core Protocol (v2.0.0)

```rust
// Proposed: Universal recurring payment primitives
#[account]
pub struct Payee {
    pub authority: Pubkey,          // 32 bytes
    pub usdc_mint: Pubkey,          // 32 bytes
    pub treasury_ata: Pubkey,       // 32 bytes
    pub platform_fee_bps: u16,      // 2 bytes
    pub bump: u8,                   // 1 byte
}
// Total: 99 bytes + 8 discriminator = 107 bytes
// Savings: 1 byte (removed tier)

#[account]
pub struct PaymentTerms {
    pub payee: Pubkey,              // 32 bytes
    pub id: [u8; 32],               // 32 bytes
    pub amount: u64,                // 8 bytes
    pub period_secs: u64,           // 8 bytes
}
// Total: 80 bytes + 8 discriminator = 88 bytes
// Savings: 40 bytes (removed grace_secs, name, active)

#[account]
pub struct PaymentAgreement {
    pub payment_terms: Pubkey,      // 32 bytes
    pub payer: Pubkey,              // 32 bytes
    pub next_payment_ts: i64,       // 8 bytes
    pub active: bool,               // 1 byte
    pub payment_count: u32,         // 4 bytes
    pub created_ts: i64,            // 8 bytes
    pub last_amount: u64,           // 8 bytes
    pub last_payment_ts: i64,       // 8 bytes
    pub bump: u8,                   // 1 byte
}
// Total: 102 bytes + 8 discriminator = 110 bytes
// Savings: 10 bytes (removed trial_ends_at, in_trial)
```

### B.3 Subscription Extension Accounts

```rust
// Extension: Subscription-specific metadata
#[account]
pub struct SubscriptionPlan {
    pub payment_terms: Pubkey,      // 32 bytes - reference to core PaymentTerms
    pub name: String,               // Variable (max 100 bytes)
    pub active: bool,               // 1 byte
    pub grace_period_secs: u64,     // 8 bytes
    pub trial_duration_secs: Option<u64>, // 9 bytes
}
// Total: ~150 bytes + 8 discriminator = ~158 bytes
// Only created for subscription use case

#[account]
pub struct SubscriptionMetadata {
    pub agreement: Pubkey,          // 32 bytes - reference to core PaymentAgreement
    pub trial_ends_at: Option<i64>, // 9 bytes
    pub in_trial: bool,             // 1 byte
    pub tier: SubscriptionTier,     // 1 byte
}
// Total: 43 bytes + 8 discriminator = 51 bytes
// Only created for subscriptions with trials/tiers
```

**Key Insight:** Users only pay rent for subscription features if they use subscriptions. Payroll, investment, and other use cases avoid this overhead.

---

## Appendix C: Event Schema Comparison

### C.1 Current Events (Subscription-Specific)

```rust
// Current: Subscription terminology throughout
#[event]
pub struct Subscribed {
    pub merchant: Pubkey,
    pub plan: Pubkey,
    pub subscriber: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Renewed {
    pub merchant: Pubkey,
    pub plan: Pubkey,
    pub subscriber: Pubkey,
    pub amount: u64,
    pub keeper: Pubkey,
    pub keeper_fee: u64,
}

#[event]
pub struct TrialStarted {         // ⚠️ Subscription-specific
    pub subscription: Pubkey,
    pub subscriber: Pubkey,
    pub plan: Pubkey,
    pub trial_ends_at: i64,
}

#[event]
pub struct TrialConverted {       // ⚠️ Subscription-specific
    pub subscription: Pubkey,
    pub subscriber: Pubkey,
    pub plan: Pubkey,
}
```

### C.2 Proposed Core Events (Universal)

```rust
// Proposed: Universal recurring payment events
#[event]
pub struct PaymentAgreementStarted {
    pub payee: Pubkey,
    pub payment_terms: Pubkey,
    pub payer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct PaymentExecuted {
    pub payee: Pubkey,
    pub payment_terms: Pubkey,
    pub payer: Pubkey,
    pub amount: u64,
    pub keeper: Pubkey,
    pub keeper_fee: u64,
    pub payment_number: u32,        // Replaces "renewal number"
}

#[event]
pub struct PaymentAgreementPaused {
    pub payee: Pubkey,
    pub payment_terms: Pubkey,
    pub payer: Pubkey,
}

#[event]
pub struct PaymentAgreementResumed {
    pub payee: Pubkey,
    pub payment_terms: Pubkey,
    pub payer: Pubkey,
    pub payments_completed: u32,    // Historical context
}
```

### C.3 Subscription Extension Events

```rust
// Extension: Subscription-specific events
#[event]
pub struct TrialStarted {
    pub agreement: Pubkey,          // Reference to core PaymentAgreement
    pub payer: Pubkey,
    pub plan: Pubkey,
    pub trial_ends_at: i64,
}

#[event]
pub struct TrialConverted {
    pub agreement: Pubkey,
    pub payer: Pubkey,
    pub plan: Pubkey,
    pub first_payment_amount: u64,
}

#[event]
pub struct SubscriptionGraceExpired {
    pub agreement: Pubkey,
    pub payer: Pubkey,
    pub plan: Pubkey,
    pub grace_ended_at: i64,
}
```

**Event Strategy:** Core emits universal payment events. Extensions emit use-case-specific events that reference core agreement PDAs.

---

## Document Metadata

**Author:** Claude Code (Anchor Architect Agent)
**Version:** 1.0.0
**Created:** 2025-11-08
**Last Updated:** 2025-11-08
**Status:** Architectural Recommendation
**Review Required:** Team architectural decision

**Related Documents:**
- `.claude/GLOBAL_DELEGATE_REFACTOR.md` - Global delegate architecture
- `program/src/state.rs` - Current account structures
- `CLAUDE.md` - Project overview and context

**Next Review:** After team decision on architectural direction
