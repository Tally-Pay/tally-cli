# Tally CLI - SDK Coverage & Implementation Plan

**Document Version**: 1.0
**Date**: 2025-11-03
**Status**: Planning Phase

## Executive Summary

This document outlines the gaps between the tally-sdk functionality and the tally-cli implementation, prioritized by merchant impact and production readiness. The CLI currently exposes **40% (8/20)** of available SDK operations, with **3 high-priority gaps** blocking full merchant functionality.

### Current State
- **SDK Coverage**: 8/20 operations (40%)
- **Production Readiness**: ⚠️ Requires fixes before merchant production use
- **Code Quality**: ✅ Excellent (no critical safety issues)
- **Architecture**: ✅ Proper SDK-first design

### Immediate Actions Required
1. Implement `update-plan-terms` command
2. Implement `close-subscription` command
3. Fix safety issue in formatting.rs (unwrap() usage)

---

## SDK Coverage Analysis

### Currently Implemented ✅

| SDK Method | CLI Command | File | Status |
|------------|-------------|------|--------|
| `init_config()` | `init-config` | `commands/init_config.rs` | ✅ Complete |
| `init_merchant()` | `init-merchant` | `commands/init_merchant.rs` | ✅ Complete |
| `create_plan()` | `create-plan` | `commands/create_plan.rs` | ✅ Complete |
| `fetch_plans()` | `list-plans` | `commands/list_plans.rs` | ✅ Complete |
| `fetch_subscriptions()` | `list-subs` | `commands/list_subs.rs` | ✅ Complete |
| `fetch_subscription_events()` | `dashboard overview` | `commands/dashboard.rs` | ✅ Complete |
| `simulate_events()` | `simulate-events` | `commands/simulate_events/mod.rs` | ✅ Complete |
| `withdraw_platform_fees()` | `withdraw-fees` | `commands/withdraw_fees.rs` | ✅ Complete (Admin) |

### Missing Functionality ❌

#### High Priority (Blocks Core Merchant Workflows)

| SDK Method | Use Case | Impact | Estimated Effort |
|------------|----------|--------|------------------|
| `update_plan_terms()` | Modify plan pricing, billing periods, grace periods | 🔴 Critical - Merchants need pricing flexibility | 4-6 hours |
| `close_subscription()` | Reclaim rent from canceled subscriptions | 🔴 Critical - User experience & cost recovery | 3-4 hours |
| `deactivate_plan()` | Disable plan from accepting new subscriptions | 🟡 High - Requires Anchor program update first | Blocked |

#### Medium Priority (Enhanced User Experience)

| SDK Method | Use Case | Impact | Estimated Effort |
|------------|----------|--------|------------------|
| `start_subscription()` | Subscribe to a plan (user flow) | 🟡 Useful for testing/demos | 4-6 hours |
| `cancel_subscription()` | Cancel active subscription (user flow) | 🟡 Useful for testing/demos | 3-4 hours |
| `renew_subscription()` | Manually trigger renewal (keeper flow) | 🟡 Useful for testing/keeper simulation | 3-4 hours |
| `fetch_merchant()` | Get merchant account details | 🟡 Useful for merchant inspection | 2-3 hours |
| `fetch_config()` | Get global config details | 🟡 Useful for platform inspection | 2-3 hours |
| `fetch_subscription()` | Get single subscription details | 🟡 Useful for debugging | 2-3 hours |

#### Low Priority (Admin & Platform Operations)

| SDK Method | Use Case | Impact | Estimated Effort |
|------------|----------|--------|------------------|
| `update_max_platform_fee()` | Adjust platform fee cap | 🟢 Platform admin only | 2-3 hours |
| `update_platform_authority()` | Change platform authority | 🟢 Platform admin only | 2-3 hours |
| `withdraw_merchant_fees()` | Withdraw merchant accumulated fees | 🟢 Could be useful for merchants | 3-4 hours |

---

## Detailed Implementation Plans

### HIGH-1: Implement `update-plan-terms` Command

**Priority**: 🔴 Critical
**Estimated Effort**: 4-6 hours
**Blocking**: Core merchant functionality

#### Problem Statement
Merchants cannot modify plan pricing, billing periods, or grace periods after initial creation. This prevents:
- Seasonal pricing adjustments
- Promotional discounts
- Billing frequency changes
- Grace period tuning based on customer behavior

#### SDK Method Signature
```rust
pub async fn update_plan_terms(
    &self,
    plan: &Pubkey,
    new_price: Option<u64>,
    new_period_seconds: Option<i64>,
    new_grace_period_seconds: Option<i64>,
) -> Result<Signature>
```

#### CLI Design
```bash
tally-cli update-plan-terms \
  --plan <PLAN_PDA> \
  [--price <USDC_MICRO_UNITS>] \
  [--period <SECONDS>] \
  [--grace-period <SECONDS>]
```

**Validation Rules**:
- At least one optional field must be provided
- All amounts use checked arithmetic
- Proper error messages for invalid values

#### Implementation Checklist
- [ ] Create `src/commands/update_plan_terms.rs`
- [ ] Define `UpdatePlanTermsRequest` struct
- [ ] Implement `execute()` function with SDK call
- [ ] Add validation for at least one field provided
- [ ] Add to `Commands` enum in `main.rs`
- [ ] Wire up in `execute_command()` match
- [ ] Write unit tests for validation
- [ ] Test against localnet deployment
- [ ] Update README with command documentation

#### Example Usage
```bash
# Update price only
tally-cli update-plan-terms \
  --plan HkDq...xyz \
  --price 15000000  # Change from $10 to $15/month

# Update billing period
tally-cli update-plan-terms \
  --plan HkDq...xyz \
  --period 5184000  # Change from 30 to 60 days

# Update multiple fields
tally-cli update-plan-terms \
  --plan HkDq...xyz \
  --price 9000000 \
  --grace-period 604800  # $9/month with 7-day grace
```

#### Testing Requirements
- Unit test: At least one field required validation
- Unit test: Individual field updates
- Unit test: Multiple field updates
- Integration test: Verify on-chain state changes

---

### HIGH-2: Implement `close-subscription` Command

**Priority**: 🔴 Critical
**Estimated Effort**: 3-4 hours
**Blocking**: Complete subscription lifecycle

#### Problem Statement
Users cannot reclaim rent (~0.00099792 SOL) from canceled subscriptions. This creates:
- Unnecessary cost for users with many old subscriptions
- Incomplete subscription lifecycle management
- Poor user experience

#### SDK Method Signature
```rust
pub async fn close_subscription(
    &self,
    subscription: &Pubkey,
    subscriber: &Pubkey,
) -> Result<Signature>
```

#### CLI Design
```bash
tally-cli close-subscription \
  --subscription <SUBSCRIPTION_PDA> \
  --subscriber <SUBSCRIBER_PUBKEY>
```

**Validation Rules**:
- Subscription must be in Canceled status
- Subscriber must match the subscription's subscriber field
- Cannot close Active subscriptions

#### Implementation Checklist
- [ ] Create `src/commands/close_subscription.rs`
- [ ] Define `CloseSubscriptionRequest` struct
- [ ] Implement `execute()` function with SDK call
- [ ] Add proper error handling for invalid states
- [ ] Add to `Commands` enum in `main.rs`
- [ ] Wire up in `execute_command()` match
- [ ] Write unit tests
- [ ] Test cancel → close flow on localnet
- [ ] Update README with command documentation

#### Example Usage
```bash
# Close a canceled subscription
tally-cli close-subscription \
  --subscription 8rPq...abc \
  --subscriber 5jKm...xyz

# Expected output
Successfully closed subscription 8rPq...abc
Reclaimed rent: ~0.00099792 SOL to 5jKm...xyz
```

#### Testing Requirements
- Unit test: Pubkey parsing
- Integration test: Cancel → close flow
- Integration test: Error when closing active subscription
- Integration test: Verify rent reclamation

---

### HIGH-3: Complete `deactivate-plan` Implementation

**Priority**: 🔴 Critical (Blocked)
**Estimated Effort**: 2-3 hours
**Status**: ⚠️ Blocked - Requires Anchor program update

#### Problem Statement
The `deactivate-plan` command exists but is not functional because the corresponding Anchor instruction has not been implemented in the program. Currently returns error:
```
Error: Instruction does not exist in program
```

#### Current State
- CLI command stub exists: `src/commands/deactivate_plan.rs`
- SDK method likely exists but untested
- Anchor program instruction missing

#### Blocking Dependencies
1. **Anchor Program**: Implement `deactivate_plan` instruction
   - Add `deactivate` field to Plan account struct
   - Create instruction handler
   - Add validation constraints
   - Write program tests

2. **SDK**: Update/test `deactivate_plan()` method
   - Verify instruction builder
   - Test against updated program

3. **CLI**: Complete implementation (currently 80% done)
   - Remove stub/placeholder logic
   - Test against updated program

#### Recommended Action
**Escalate to Anchor program team** - This is blocked on program-level implementation.

---

### MED-1: Implement Subscriber Flow Commands

**Priority**: 🟡 Medium
**Estimated Effort**: 10-14 hours (all three commands)
**Use Case**: Testing, demos, end-to-end workflows

While merchants primarily need merchant-focused commands, having subscriber commands enables:
- **Local testing** without building separate subscriber tools
- **Demo workflows** for presentations and onboarding
- **Integration testing** for full subscription lifecycle
- **Troubleshooting** by simulating user actions

#### Commands to Implement

##### 1. `start-subscription`
```bash
tally-cli start-subscription \
  --plan <PLAN_PDA> \
  --subscriber <SUBSCRIBER_PUBKEY> \
  --token-account <USDC_ATA> \
  --delegate-amount <USDC_MICRO_UNITS>
```

**SDK Method**: `start_subscription()`
**Effort**: 4-6 hours

##### 2. `cancel-subscription`
```bash
tally-cli cancel-subscription \
  --subscription <SUBSCRIPTION_PDA> \
  --subscriber <SUBSCRIBER_PUBKEY> \
  --token-account <USDC_ATA>
```

**SDK Method**: `cancel_subscription()`
**Effort**: 3-4 hours

##### 3. `renew-subscription`
```bash
tally-cli renew-subscription \
  --subscription <SUBSCRIPTION_PDA> \
  --keeper <KEEPER_PUBKEY>
```

**SDK Method**: `renew_subscription()`
**Effort**: 3-4 hours

#### Implementation Notes
- These commands should be clearly documented as testing/demo tools
- Consider adding a `--confirm` flag for subscriber operations
- Add warnings about delegate security implications
- Include simulation/dry-run mode

---

### MED-2: Implement Account Inspection Commands

**Priority**: 🟡 Medium
**Estimated Effort**: 6-9 hours (all commands)
**Use Case**: Debugging, inspection, monitoring

#### Commands to Implement

##### 1. `show-merchant`
```bash
tally-cli show-merchant \
  --merchant <MERCHANT_PDA> \
  [--output json|human]
```

**SDK Method**: `fetch_merchant()`
**Effort**: 2-3 hours

**Output**:
```
Merchant: HkDq...xyz
Authority: 5jKm...abc
Treasury: 9rTp...def (USDC ATA)
Fee Rate: 1.50% (150 bps)
Status: Active
Total Revenue: $12,345.67
```

##### 2. `show-config`
```bash
tally-cli show-config \
  [--output json|human]
```

**SDK Method**: `fetch_config()`
**Effort**: 2-3 hours

**Output**:
```
Global Configuration
====================
Program ID: eUV3...d7
Platform Authority: 6jsd...JM5
Max Platform Fee: 10.00% (1000 bps)
Keeper Fee: 0.50% (50 bps)
Platform Treasury: 8rPq...xyz
```

##### 3. `show-subscription`
```bash
tally-cli show-subscription \
  --subscription <SUBSCRIPTION_PDA> \
  [--output json|human]
```

**SDK Method**: `fetch_subscription()`
**Effort**: 2-3 hours

**Output**:
```
Subscription: 8rPq...abc
Plan: HkDq...xyz (Premium Plan)
Subscriber: 5jKm...def
Status: Active
Started: 2025-10-15 14:23:45 UTC
Next Renewal: 2025-11-15 14:23:45 UTC (in 12 days)
Total Paid: $60.00 (6 renewals)
```

---

### LOW-1: Implement Admin Platform Commands

**Priority**: 🟢 Low
**Estimated Effort**: 6-9 hours
**Use Case**: Platform administration
**Security**: Should be behind feature flag or separate admin binary

#### Commands to Implement

##### 1. `update-max-platform-fee`
```bash
tally-cli update-max-platform-fee \
  --new-max-bps <BASIS_POINTS>
```

**SDK Method**: `update_max_platform_fee()`
**Effort**: 2-3 hours

##### 2. `update-platform-authority`
```bash
tally-cli update-platform-authority \
  --new-authority <PUBKEY>
```

**SDK Method**: `update_platform_authority()`
**Effort**: 2-3 hours

##### 3. `withdraw-merchant-fees`
```bash
tally-cli withdraw-merchant-fees \
  --merchant <MERCHANT_PDA> \
  --destination <USDC_ATA>
```

**SDK Method**: `withdraw_merchant_fees()`
**Effort**: 3-4 hours

#### Security Considerations
- These commands require platform authority signatures
- Should not be exposed in standard merchant CLI
- Consider separate `tally-admin-cli` binary
- Or implement behind `--admin` feature flag

---

## Code Quality Issues

### SAFETY-1: Replace `unwrap()` in formatting.rs

**Priority**: 🔴 High
**File**: `/home/rodzilla/projects/tally/tally-cli/src/utils/formatting.rs:78`
**Effort**: 30 minutes

#### Issue
```rust
pub fn format_timestamp_human(timestamp: i64) -> String {
    let datetime = DateTime::from_timestamp(timestamp, 0).unwrap(); // Line 78
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
```

**Risk**: Panic if timestamp is out of valid range (years 262144 BCE to 262143 CE).

#### Fix
```rust
pub fn format_timestamp_human(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| format!("Invalid timestamp: {}", timestamp))
}
```

#### Testing
- Add unit test with valid timestamp (e.g., 1698752625)
- Add unit test with out-of-range timestamp
- Verify error message format

---

### QUALITY-1: Improve Error Context Consistency

**Priority**: 🟡 Medium
**Effort**: 2-3 hours

#### Issue
Some commands lack detailed error context using `anyhow::Context`.

#### Example Improvements

**Before**:
```rust
let merchant = Pubkey::from_str(merchant_str)?;
```

**After**:
```rust
let merchant = Pubkey::from_str(merchant_str)
    .context("Failed to parse merchant public key")?;
```

#### Files to Update
- `commands/create_plan.rs`
- `commands/list_plans.rs`
- `commands/list_subs.rs`
- `commands/dashboard.rs`

#### Pattern to Follow
```rust
use anyhow::{Context, Result};

// Parsing
.context("Failed to parse <field> as <type>")?

// SDK calls
.context("Failed to <operation> - check RPC connection and account state")?

// I/O operations
.context("Failed to write output to <destination>")?
```

---

### QUALITY-2: Consistent Output Format Handling

**Priority**: 🟡 Medium
**Effort**: 2-3 hours

#### Issue
Some commands handle `--output json` flag inconsistently or don't support it at all.

#### Recommendation
Create a helper trait:

```rust
pub trait JsonSerializable {
    fn to_json(&self) -> Result<String>;
}

pub fn format_output<T: JsonSerializable + std::fmt::Display>(
    data: &T,
    format: OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Json => data.to_json(),
        OutputFormat::Human => Ok(data.to_string()),
    }
}
```

#### Apply to Commands
- `list-plans`
- `list-subs`
- `dashboard`
- All new inspection commands

---

## Implementation Roadmap

### Phase 1: Critical Fixes (Week 1)
**Goal**: Production-ready for core merchant workflows

| Task | Priority | Effort | Assignee | Status |
|------|----------|--------|----------|--------|
| Implement `update-plan-terms` | 🔴 Critical | 4-6h | - | ✅ Complete |
| Implement `close-subscription` | 🔴 Critical | 3-4h | - | ✅ Complete |
| Fix unwrap() in formatting.rs | 🔴 High | 30m | - | ✅ Complete |
| Escalate `deactivate-plan` to program team | 🔴 Critical | N/A | - | ⬜ Pending |

**Deliverable**: CLI v0.2.0 with 10/20 SDK operations (50% coverage)

---

### Phase 2: Enhanced Testing (Week 2)
**Goal**: Complete testing and demo capabilities

| Task | Priority | Effort | Assignee | Status |
|------|----------|--------|----------|--------|
| Implement `start-subscription` | 🟡 Medium | 4-6h | - | ⬜ Pending |
| Implement `cancel-subscription` | 🟡 Medium | 3-4h | - | ⬜ Pending |
| Implement `renew-subscription` | 🟡 Medium | 3-4h | - | ⬜ Pending |
| Implement inspection commands | 🟡 Medium | 6-9h | - | ⬜ Pending |
| Improve error context | 🟡 Medium | 2-3h | - | ⬜ Pending |

**Deliverable**: CLI v0.3.0 with 16/20 SDK operations (80% coverage)

---

### Phase 3: Admin & Polish (Week 3)
**Goal**: Complete SDK coverage and production polish

| Task | Priority | Effort | Assignee | Status |
|------|----------|--------|----------|--------|
| Implement admin commands | 🟢 Low | 6-9h | - | ⬜ Pending |
| Consistent output formatting | 🟡 Medium | 2-3h | - | ⬜ Pending |
| Complete `deactivate-plan` | 🔴 Critical | 2-3h | - | ⚠️ Blocked |
| Comprehensive documentation | 🟡 Medium | 3-4h | - | ⬜ Pending |

**Deliverable**: CLI v1.0.0 with 20/20 SDK operations (100% coverage)

---

## Testing Strategy

### Unit Tests
Each new command must include:
- [ ] Input validation tests
- [ ] Pubkey parsing tests
- [ ] Error case handling tests
- [ ] Output formatting tests

### Integration Tests
Test against localnet deployment:
- [ ] Happy path: Create merchant → create plan → update plan
- [ ] Subscription lifecycle: Start → renew → cancel → close
- [ ] Error cases: Invalid accounts, wrong signers
- [ ] Edge cases: Boundary values, timing issues

### End-to-End Testing Checklist
```bash
# Merchant workflow
tally-cli init-merchant --treasury <ATA> --fee-bps 150
tally-cli create-plan --merchant <PDA> --id "test" --price 10000000 --period 2592000
tally-cli list-plans --merchant <PDA>
tally-cli update-plan-terms --plan <PDA> --price 15000000
tally-cli show-merchant --merchant <PDA>

# Subscriber workflow (testing)
tally-cli start-subscription --plan <PDA> --subscriber <KEY> --token-account <ATA>
tally-cli show-subscription --subscription <PDA>
tally-cli renew-subscription --subscription <PDA> --keeper <KEY>
tally-cli cancel-subscription --subscription <PDA> --subscriber <KEY>
tally-cli close-subscription --subscription <PDA> --subscriber <KEY>

# Admin workflow
tally-cli withdraw-fees --destination <ATA>
tally-cli show-config
```

---

## Documentation Requirements

### README Updates
For each new command, add:
- Command syntax
- Required arguments
- Optional flags
- Example usage
- Expected output
- Common errors

### CLAUDE.md Updates
Update command examples section with:
- New merchant operations
- Subscriber flow examples
- Admin command usage
- Testing patterns

### Inline Documentation
Each command module must have:
- Module-level doc comment explaining purpose
- Function doc comments with `# Arguments` and `# Returns`
- Error case documentation
- Example usage in doc comments

---

## Success Metrics

### Coverage Goals
- **Phase 1**: 50% SDK coverage (10/20 operations)
- **Phase 2**: 80% SDK coverage (16/20 operations)
- **Phase 3**: 100% SDK coverage (20/20 operations)

### Quality Gates
- [ ] Zero clippy warnings
- [ ] Zero unsafe code blocks
- [ ] Zero unwrap/expect in production code
- [ ] >80% test coverage on new code
- [ ] All integration tests pass
- [ ] Documentation complete

### Production Readiness Criteria
- [ ] All critical merchant operations implemented
- [ ] Complete subscription lifecycle support
- [ ] Comprehensive error handling
- [ ] User-friendly error messages
- [ ] Full test coverage
- [ ] Documentation complete
- [ ] Security review complete

---

## Risk Assessment

### High Risk
- **Blocked `deactivate-plan`**: Requires cross-team coordination with program developers
  - Mitigation: Escalate immediately, track in separate issue

### Medium Risk
- **SDK API Changes**: SDK updates may break CLI integration
  - Mitigation: Pin SDK version, test thoroughly before updates

### Low Risk
- **Testing Coverage**: New commands may have insufficient testing
  - Mitigation: Mandatory integration tests for all new commands

---

## Appendix

### Reference: SDK Method Inventory

Complete list from `tally-sdk/src/simple_client.rs` and `transaction_builder.rs`:

**Global Config Operations** (2):
1. ✅ `init_config()` - Initialize global config
2. ❌ `update_max_platform_fee()` - Update platform fee cap
3. ❌ `update_platform_authority()` - Change platform authority

**Merchant Operations** (3):
4. ✅ `init_merchant()` - Initialize merchant
5. ❌ `fetch_merchant()` - Get merchant details
6. ❌ `withdraw_merchant_fees()` - Withdraw merchant balance

**Plan Operations** (5):
7. ✅ `create_plan()` - Create subscription plan
8. ❌ `update_plan_terms()` - Update plan pricing/terms
9. ❌ `deactivate_plan()` - Disable plan (blocked on program)
10. ✅ `fetch_plans()` - List merchant plans

**Subscription Operations** (6):
11. ❌ `start_subscription()` - Subscribe to plan
12. ❌ `renew_subscription()` - Process renewal
13. ❌ `cancel_subscription()` - Cancel subscription
14. ❌ `close_subscription()` - Reclaim rent
15. ✅ `fetch_subscriptions()` - List subscriptions
16. ❌ `fetch_subscription()` - Get subscription details

**Event & Analytics Operations** (2):
17. ✅ `fetch_subscription_events()` - Get events
18. ✅ `simulate_events()` - Simulate event stream

**Admin Operations** (2):
19. ✅ `withdraw_platform_fees()` - Platform fee withdrawal
20. ❌ (Future) Platform analytics/reporting

---

## Document Maintenance

**Last Updated**: 2025-11-03
**Next Review**: After Phase 1 completion
**Owner**: Tally CLI Team

### Change Log
- 2025-11-03: Initial document created from code-reviewer agent findings
