# Updated Implementation Plan: Firm Trading Conflict Detection

**Date**: 2026-01-23
**Status**: Ready for Implementation

## Summary

1. **3-Tier Lookup (Phase 2)**: ✅ Already complete - no work needed
2. **Full Desk Name Resolution Chain**: 4 tables exist in dev
3. **Position Size Logic**: Confirmed mapping:
   - `position_size > 0` → LONG → "Buy"
   - `position_size < 0` → SHORT → "Sell"
   - `position_size = 0 or NULL` → FLAT → "-"

---

## Pre-requisite: Database Permissions

```sql
-- Grant SELECT on new tables (run as DBA):
GRANT SELECT ON bo_airflow.oracle_portfolio_meta_data TO bo_airflow;
GRANT SELECT ON bo_airflow.oracle_cost_centre TO bo_airflow;
```

---

## Desk Name Resolution Chain

```
ProductUsage / OraclePosition
    │ portfolio
    ▼
OraclePortfolioGroup
    │ rp_portfolio_id → id
    ▼
OraclePortfolioMetaData
    │ cost_centre → cost_centre
    ▼
OracleCostCentre
    │ display_name ← GOLD STANDARD
```

**Fallback Priority:**
1. `oracle_cost_centre.display_name` - "US Equity Desk" (gold standard)
2. `oracle_portfolio_meta_data.pnl_cost_centre_name` - "ARB_US"
3. `oracle_portfolio_meta_data.desk` - desk field
4. `oracle_portfolio_group.rp_portfolio` - final fallback

---

## Phase 1: Data Layer (New Models)

### Task 1.1: Create OraclePortfolioGroup Model
**File**: `src/pa_dealing/db/models/market.py`

```python
class OraclePortfolioGroup(Base):
    """Portfolio grouping - links portfolios to metadata.

    Join chain: Position.portfolio → PortfolioGroup.portfolio
    Then: PortfolioGroup.rp_portfolio_id → PortfolioMetaData.id

    CRITICAL: READ-ONLY table in bo_airflow schema.
    """

    __tablename__ = "oracle_portfolio_group"
    __table_args__ = {'schema': 'bo_airflow'}

    id: Mapped[int] = mapped_column(BigInteger, primary_key=True)
    portfolio_id: Mapped[float | None] = mapped_column(Float)
    portfolio: Mapped[str | None] = mapped_column(String(100), index=True)
    rp_portfolio: Mapped[str | None] = mapped_column(String(100))
    rp_portfolio_id: Mapped[float | None] = mapped_column(Float)
    company: Mapped[str | None] = mapped_column(String(50))
    account_name: Mapped[str | None] = mapped_column(String(100))
    inst_symbol: Mapped[str | None] = mapped_column(String(30), index=True)
```

### Task 1.2: Create OraclePortfolioMetaData Model
**File**: `src/pa_dealing/db/models/market.py`

```python
class OraclePortfolioMetaData(Base):
    """Portfolio metadata - contains desk and cost centre info.

    Join: PortfolioGroup.rp_portfolio_id → PortfolioMetaData.id
    Then: PortfolioMetaData.cost_centre → CostCentre.cost_centre

    CRITICAL: READ-ONLY table in bo_airflow schema.
    """

    __tablename__ = "oracle_portfolio_meta_data"
    __table_args__ = {'schema': 'bo_airflow'}

    id: Mapped[int] = mapped_column(BigInteger, primary_key=True)
    asset_class: Mapped[str | None] = mapped_column(String(50))
    asset_subclass: Mapped[str | None] = mapped_column(String(50))
    strategy_type: Mapped[str | None] = mapped_column(String(50))
    desk: Mapped[str | None] = mapped_column(String(100))
    cost_centre: Mapped[str | None] = mapped_column(String(50), index=True)
    pnl_cost_centre_name: Mapped[str | None] = mapped_column(String(100))
    strategy: Mapped[str | None] = mapped_column(String(100))
```

### Task 1.3: Create OracleCostCentre Model
**File**: `src/pa_dealing/db/models/market.py`

```python
class OracleCostCentre(Base):
    """Cost centre reference - contains display_name (gold standard).

    Join: PortfolioMetaData.cost_centre → CostCentre.cost_centre

    CRITICAL: READ-ONLY table in bo_airflow schema.
    """

    __tablename__ = "oracle_cost_centre"
    __table_args__ = {'schema': 'bo_airflow'}

    id: Mapped[int] = mapped_column(BigInteger, primary_key=True)
    cost_centre: Mapped[str | None] = mapped_column(String(50), unique=True, index=True)
    description: Mapped[str | None] = mapped_column(String(200))
    display_name: Mapped[str | None] = mapped_column(String(100))  # GOLD STANDARD
    is_deleted_yn: Mapped[str | None] = mapped_column(String(1))
```

### Task 1.4: Export in models/__init__.py
Add all new models to exports.

---

## Phase 3: Conflict Detection Enhancement

### Task 3.1: Enhance get_mako_position_info()
**File**: `src/pa_dealing/db/repository.py`

Enhanced query with full desk name resolution:

```python
async def get_mako_position_info(
    session: AsyncSession,
    inst_symbol: str,
) -> MakoPositionInfo | None:
    """Get Mako firm trading activity with full desk name resolution.

    Data sources:
    - ProductUsage: last_trade_date, last_position_date
    - OraclePosition: position_size (for firm direction)
    - Full desk chain: PortfolioGroup → PortfolioMetaData → CostCentre

    Position Size → Direction Mapping:
    - position_size > 0  → LONG  → "Buy"
    - position_size < 0  → SHORT → "Sell"
    - position_size = 0/NULL → FLAT → "-"

    TODO: For actual historical trade size/direction, query
    historic_holding_trade table. Current logic uses oracle_position
    (current snapshot) as proxy for firm direction.
    """
```

### Task 3.2: Desk Name Resolution Helper
```python
def _resolve_desk_name(
    cost_centre: OracleCostCentre | None,
    meta_data: OraclePortfolioMetaData | None,
    portfolio_group: OraclePortfolioGroup | None,
) -> str | None:
    """Resolve desk name using fallback chain.

    Priority:
    1. cost_centre.display_name - "US Equity Desk" (gold standard)
    2. meta_data.pnl_cost_centre_name - "ARB_US"
    3. meta_data.desk - desk field
    4. portfolio_group.rp_portfolio - final fallback
    """
    if cost_centre and cost_centre.display_name:
        return cost_centre.display_name
    if meta_data and meta_data.pnl_cost_centre_name:
        return meta_data.pnl_cost_centre_name
    if meta_data and meta_data.desk:
        return meta_data.desk
    if portfolio_group and portfolio_group.rp_portfolio:
        return portfolio_group.rp_portfolio
    return None
```

### Task 3.3: Update calculate_conflict_risk()
**Position Size Logic** (already implemented, verify correct):
```python
if mako_info.position_size is not None:
    if mako_info.position_size > 0:
        firm_direction = "LONG"  # Maps to "Buy"
    elif mako_info.position_size < 0:
        firm_direction = "SHORT"  # Maps to "Sell"
    else:
        firm_direction = "FLAT"  # Maps to "-" (No Direction)
```

---

## Phase 5: Slack UI Enhancement

### Task 5.1: Add Conflict Fields to SlackMessageRequest
**File**: `src/pa_dealing/agents/slack/schemas.py`

```python
class SlackMessageRequest(BaseModel):
    # ... existing fields ...

    # NEW: Conflict detection fields
    has_conflict: bool = False
    conflict_level: str | None = None  # high, medium, low, none
    conflict_desk: str | None = None  # e.g., "US Equity Desk"
    days_since_mako_trade: int | None = None
    firm_direction: str | None = None  # LONG, SHORT, FLAT
    risk_factors: list[str] = Field(default_factory=list)
```

### Task 5.2: Update build_approval_request_blocks()
**File**: `src/pa_dealing/agents/slack/ui.py`

Target layout:
```
┌─────────────────────────────────────────────────────┐
│  ⚖️ Compliance Approval Required                   │
├─────────────────────────────────────────────────────┤
│ *Requester:* John Smith    *Request ID:* PAD-001   │
│ *Security:* Apple Inc      *Action:* BUY 100       │
│ *Est. Value:* USD 15,000   *Risk Level:* 🔴 HIGH   │
├─────────────────────────────────────────────────────┤
│ ⚠️ *Conflict Alert*                                │
│ • Mako traded AAPL 5 days ago                      │
│ • Desk: US Equity Desk                             │
│ • Direction: Employee BUY ↔ Mako LONG (High Risk)  │
├─────────────────────────────────────────────────────┤
│ *Risk Factors*                                     │
│ • Mako traded this security 5 days ago             │
│ • Employee BUY aligns with firm LONG position      │
│ • Active in desk: US Equity Desk                   │
├─────────────────────────────────────────────────────┤
│ *Justification*                                    │
│ > Long-term investment                             │
├─────────────────────────────────────────────────────┤
│ ℹ️ Approved trades must be executed within 2       │
│    business days per PA Dealing Policy v18.5       │
├─────────────────────────────────────────────────────┤
│ [Approve]  [Decline]                               │
└─────────────────────────────────────────────────────┘
```

### Task 5.3: Conflict Warning Block Builder
### Task 5.4: Risk Factors Block Builder
### Task 5.5: 2-Day Execution Reminder Block
### Task 5.6: Wire up SlackMessageRequest population

---

## Implementation Order

1. **Pre-req**: Grant DB permissions (~DBA task)
2. **Phase 1.1-1.4**: Create 3 new models (~20 min)
3. **Phase 3.1-3.3**: Enhance conflict detection with JOINs (~45 min)
4. **Phase 5.1**: Add fields to SlackMessageRequest (~10 min)
5. **Phase 5.2-5.6**: Update UI blocks and wiring (~40 min)
6. **Testing**: Manual UAT with Slack bot (~15 min)

**Total estimated time**: ~2.5 hours

---

## Verification Checklist

- [ ] DB permissions granted for oracle_portfolio_meta_data and oracle_cost_centre
- [ ] All 3 new models created and exported
- [ ] get_mako_position_info() returns desk_name (via full chain) and position_size
- [ ] calculate_conflict_risk() correctly maps position_size to direction
- [ ] Slack approval notifications show conflict warning when has_conflict=True
- [ ] Risk factors displayed as bullet list
- [ ] 2-day execution reminder shown
- [ ] Dashboard shows same conflict info (Phase 6 - later)
