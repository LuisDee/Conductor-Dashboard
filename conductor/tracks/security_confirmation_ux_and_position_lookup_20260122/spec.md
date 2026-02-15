# Track Specification: Security Confirmation UX & Position Lookup

**Track ID**: `security_confirmation_ux_and_position_lookup_20260122`
**Type**: Bug Fixes + Feature Enhancement
**Priority**: HIGH (blocks trade submission + improves compliance detection)
**Estimated Complexity**: Medium-High

---

## Overview

Based on UAT testing on January 22, 2026, we identified 5 critical issues that need to be addressed to improve the PA Dealing agent's security confirmation flow, fix blocking bugs, and add compliance risk detection capabilities.

---

## Problem Statement

### Issue #1: Database Error (CRITICAL - Blocks Trade Submission)
**Error**: `column oracle_position.last_trade_date does not exist`

**Impact**: Users cannot complete trade submissions. When user confirms "yes" to submit trade, the system crashes.

**Root Cause**:
- `OraclePosition` model defines `last_trade_date` column
- Alembic migration exists but was never run on dev database
- SQLAlchemy tries to SELECT column that doesn't exist in physical schema

**Current Behavior**: Trade submission fails with database error

**Desired Behavior**: Trade submission succeeds

---

### Issue #2: Symbol Extraction (UX Issue)
**Problem**: LLM extracts full phrase including derivative types

**Example**:
```
User: "I want to buy 5 lots of bund calls @ 100"
Current: LLM extracts "bund calls"
         Lookup fails: "couldn't find 'bund calls'"
Desired: LLM extracts "bund"
         Lookup succeeds: "Can you confirm BUND?"
```

**Impact**: Users have to re-type security name, poor UX

**Root Cause**: Minimal term cleaning only removes negation words, doesn't strip:
- Derivative types: calls, puts, options, futures, forwards
- Quantity words: lots, shares, units, contracts
- Price indicators: @ 100, at 50.25

---

### Issue #3: Confirmation UX (UX Improvement)
**Problem**: Bot shows numbered list of all matches, overwhelming

**Current Behavior**:
```
Bot: I found several matches for 'bund':
⭐ 1. BUND - Euro Bond [mapping]
2. BOBLI - Bundesolbligation I/L [product]
3. VER AV - Verbund AG [bloomberg]
4. BUNDI - Euro Bond Inflation Linked [mapping]

Is that correct? Reply 'yes', a number (1-4), or the ticker to confirm.
```

**Desired Behavior**:
```
Bot: Can you confirm you are referring to BUND (Euro Bond)?
```

**User says "no"**:
```
Bot: What security are you looking for?

(Similar matches you might mean:
 • BOBLI - Bundesolbligation I/L
 • BUNDI - Euro Bond Inflation Linked
 • VER AV - Verbund AG)
```

**Impact**: Simpler, cleaner UX. Still allows user to reject and see alternatives.

---

### Issue #4: Position Lookup & Conflict Detection (NEW FEATURE)
**Problem**: System doesn't check if Mako recently traded the same security (front-running risk)

**Business Requirement**: PA Dealing Policy - 30-Day Conflict Window
- If Mako traded a security within last 30 days, flag as potential conflict
- Compliance needs to review for front-running risk

**Current Behavior**: No conflict detection

**Desired Behavior**:
1. When employee requests trade on security X:
   - Query `product_usage` table for Mako's recent trading activity
   - Get `last_trade_date`, `last_position_date`, firm direction (buy/sell)
   - Calculate if within 30-day window
   - Query employee's own approved trades on same security

2. Display conflict warning to Compliance:
   ```
   ⚠️ Conflict Detected:
   - Mako last traded BUND: 15 days ago
   - Mako position: LONG (buying)
   - Employee direction: BUY
   - Employee's prior BUND trades: 2 approved in last 6 months
   ```

**Data Sources**:

**A. Mako Firm Trading** (from `bo_airflow` schema):
- `product_usage` table:
  - `inst_symbol` (security identifier)
  - `portfolio` (trading book)
  - `last_trade_date` (when Mako last traded it)
  - `last_position_date` (when Mako last held position)

- Subquery to `position` table (using `last_position_date`):
  - `position_size` (positive = long/buying, negative = short/selling)

- Subquery to `portfolio_meta_data` + `portfolio_group`:
  - `display_name` as `mako_desk` (e.g., "Fixed Income", "Commodities")

**B. Employee Trading History** (from `padealing` schema):
- `pad_request` joined with `pad_approval`:
  - Filter: `status IN ('approved', 'executed')`
  - Filter: `deleted_at IS NULL`
  - Filter: `inst_symbol = <current_security>`
  - Get: `direction` (BUY/SELL), `created_at`, `quantity`, `estimated_value`

**Conflict Detection Logic**:
```python
# Calculate days since Mako last traded
days_since_trade = (CURRENT_DATE - last_trade_date).days

# Flag as conflict if within 30 days
is_conflict = days_since_trade <= 30

# Determine firm direction
if mako_position_size > 0:
    firm_direction = "LONG (buying)"
elif mako_position_size < 0:
    firm_direction = "SHORT (selling)"
else:
    firm_direction = "FLAT (no position)"
```

---

### Issue #5: Dashboard Not Starting (UAT Script Issue)
**Problem**: `run_uat_dev_simple.sh` only starts API + Slack, not dashboard

**Current Behavior**:
- Visit http://localhost:3000/
- See: "Error loading dashboard: Network Error"

**Desired Behavior**:
- Dashboard starts automatically in tmux window 2
- Accessible at http://localhost:3000/

**Root Cause**: Script only creates 2 tmux windows (API, Slack), doesn't start dashboard

---

## Functional Requirements

### FR1: Database Error Fix
- Remove `last_trade_date` field from `OraclePosition` model
- Verify model change doesn't break existing queries
- Trade submission must complete successfully

### FR2: Symbol Extraction Enhancement
- Strip derivative types (calls, puts, options, futures, forwards)
- Strip quantity words (lots, shares, units, contracts)
- Strip price indicators (@ 100, at 50.25)
- Strip action verbs (buy, sell, buying, selling, trade, trading)
- Preserve existing negation word removal
- Test cases must pass:
  - "buying 5 lots of bund calls @ 100" → "bund"
  - "sell 10 shares of AAPL" → "aapl"
  - "trade FGBL futures" → "fgbl"

### FR3: Simplified Confirmation UX
- Show only top match by default
- Accept "yes" to confirm
- Accept "no" to show alternatives and ask for clarification
- Accept direct symbol name to override
- Remove numbered selection requirement

### FR4: Position Lookup & Conflict Detection
- Query `product_usage` for Mako trading activity
- Query `pad_request` + `pad_approval` for employee history
- Calculate conflict risk (30-day window)
- Store conflict data in request
- Display conflict warning in dashboard
- Show detailed conflict info: days since trade, firm direction, desk name, employee history count

### FR5: Dashboard Startup
- Add tmux window 2 for dashboard
- Set VITE_API_URL environment variable
- Run npm install + npm run dev
- Update script output to show dashboard URL

---

## Non-Functional Requirements

### NFR1: Performance
- Position lookup queries must complete in <500ms
- No N+1 query issues in conflict detection

### NFR2: Test Coverage
- Integration test coverage >80%
- No regressions in existing tests
- All 5 issues must have corresponding tests

### NFR3: Data Quality
- False positive rate for conflicts <10%
- Audit trail preserved for position lookups

---

## Acceptance Criteria

### Must Have
- ✅ Trade submissions succeed (no database errors)
- ✅ Symbol extraction strips derivative terms correctly
- ✅ Confirmation shows single match by default
- ✅ Conflicts detected for Mako trades <30 days
- ✅ Dashboard loads at http://localhost:3000/
- ✅ All integration tests pass
- ✅ Code coverage >80%

### Should Have
- ✅ Position lookup cached within session (if needed)
- ✅ Conflict badge color-coded in dashboard
- ✅ UAT guide updated with conflict detection tests

---

## Out of Scope

**Deferred to Future Tracks**:
- Desk/division proximity scoring
- Advanced risk scoring algorithms
- Automatic blocking of high-risk trades
- Historical pattern analysis
- Machine learning risk models
- Timeline visualization of trades
- Interactive conflict resolution workflow

---

## Technical Decisions

### Decision 1: Remove `last_trade_date` vs Run Migration
**Chosen**: Remove from model
**Rationale**:
- Column is never used (always returns None)
- Avoids migration complexity in dev/UAT environments
- Can add back later when implementing actual feature

### Decision 2: Query `product_usage` vs `oracle_position`
**Chosen**: Use `product_usage`
**Rationale**:
- Pre-computed `last_trade_date` and `last_position_date`
- Optimized for snapshot queries
- Mirrors TD-1751 pattern
- Avoids scanning millions of position records

### Decision 3: 30-Day Window Only
**Chosen**: Implement basic conflict detection only
**Rationale**:
- Clear regulatory requirement
- Keeps track focused and testable
- Future track can layer on advanced scoring

### Decision 4: Display Conflicts, Don't Block
**Chosen**: Show in dashboard, don't prevent submission
**Rationale**:
- Conflicts need human review
- Compliance makes final decision
- Some conflicts are acceptable
- Aligns with PA Dealing Policy

---

## Dependencies

### External Systems
- `bo_airflow.product_usage` table (must exist)
- `bo_airflow.position` table (must exist)
- `bo_airflow.portfolio_meta_data` table (must exist)
- `bo_airflow.portfolio_group` table (must exist)

### Internal Services
- Database connection to dev environment
- Slack bot running (for UX testing)
- Dashboard npm dependencies installed

---

## Assumptions

1. **Database Schema Stable**: `product_usage` table schema matches documentation
2. **Data Availability**: Dev database has sample data for testing
3. **PAD Tables**: Follow new normalized schema
4. **Employee IDs**: Match across all tables
5. **inst_symbol Consistency**: Same values used across all tables

---

## Timeline Estimate

- **Phase 1** (Database Fix): 1 hour
- **Phase 2** (Symbol Extraction): 2 hours
- **Phase 3** (Confirmation UX): 3 hours
- **Phase 4** (Position Lookup): 8 hours
- **Phase 5** (Dashboard Startup): 1 hour
- **Phase 6** (Testing & Docs): 4 hours

**Total**: ~19 hours (~2-3 days)
