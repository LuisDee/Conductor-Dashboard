# Specification: Firm Trading Conflict Detection & Enrichment

## Overview

Implement automated conflict-of-interest detection by analyzing Mako's firm trading activity against employee personal trading requests. This enriches the risk assessment process with firm trading context, enabling compliance officers to identify potential conflicts where employees trade securities that the firm actively manages.

**Priority**: High - Post-migration enrichment (Phase 2)
**Type**: Feature Enhancement
**Dependencies**:
- Multi-Environment Database Migration (track: `db_migration_20251230`)
- Advanced Instrument Validation (track: `advanced_instrument_validation_20260115`)

## Context

During migration planning, we identified that the legacy PA dealing system performs comprehensive conflict detection by cross-referencing employee trades against firm trading activity. Our current implementation lacks this capability.

**Current State**:
- ✅ Basic restricted list checks (`oracle_bloomberg.is_restricted`)
- ✅ Risk scoring based on security characteristics
- ❌ **No firm trading conflict detection**

**Gap Analysis**:
- Missing table: `ProductUsage` (firm trading history)
- Missing model: `OraclePortfolioGroup` (desk/portfolio context)
- Missing table: `HistoricPosition` (position details with settlement dates)
- Missing logic: Conflict detection integration in risk engine

## Objectives

1. **Add Missing Data Sources**: Integrate ProductUsage, OraclePortfolioGroup, and mock HistoricPosition data
2. **Implement Conflict Detection**: Check if Mako actively trades securities requested by employees
3. **Enrich Risk Assessment**: Add conflict flags and context to PADRequest before approval routing
4. **Advisory-Only Approach**: Flag conflicts for compliance review, NEVER auto-reject trades
5. **Complete 3-Tier Lookup**: Finish Advanced Instrument Validation track integration

## Functional Requirements

### FR-1: Data Source Integration

**FR-1.1: ProductUsage Table Sync**
- Add `product_usage` table to bo_airflow daily sync
- Fields: `id`, `inst_symbol`, `inst_type`, `company`, `portfolio`, `last_trade_date`, `last_position_date`
- Create SQLAlchemy model: `OracleProductUsage`
- Schema reference: `/home/coder/repos/bodev/backoffice-web/database_models/models/tables/table_models.py:5448-5461`

**FR-1.2: OraclePortfolioGroup Model**
- Table already exists in dev: `bo_airflow.oracle_portfolio_group`
- Create SQLAlchemy model mapping
- Fields needed: `portfolio`, `cost_centre`, `rp_portfolio`, `company`
- Schema reference: `/home/coder/repos/bodev/bodev_backend/backoffice-db-models/backoffice_db_models/models.py:4795-4823`

**FR-1.3: HistoricPosition Mock Data**
- Create schema: `OracleHistoricPosition` (read-only model)
- Fields needed (6 of 60+): `portfolio_id`, `inst_symbol`, `inst_type`, `settlement_date`, `position_size`, `exchange_id`
- Mock data: 1 week sample (~1,000-10,000 rows) for local/dev testing
- Schema reference: `/home/coder/repos/bodev/backoffice-web/database_models/models/views/views_models.py:695-754`
- **Future**: Connect to real archive table (separate track)

### FR-2: Conflict Detection Logic

**FR-2.1: Firm Trading Check**
- Query `ProductUsage` for employee's requested security
- Filter: `last_trade_date >= CURRENT_DATE - 90` (active in last 90 days)
- Return: List of portfolios/desks trading this security

**FR-2.2: Conflict Assessment**
- Integrate check into risk engine: `src/pa_dealing/services/risk.py`
- Set `PADRequest.has_conflict = True` if firm trades security
- Populate `PADRequest.conflict_comments` with:
  - Number of portfolios trading
  - Desk names (via OraclePortfolioGroup → cost_centre)
  - Last trade date
- Reference logic: `/home/coder/repos/bodev/backoffice-web/compliance_portal/pages/pa_dealing/rest.py:770-808`

**FR-2.3: Advisory Display**
- **Slack Bot**: Show conflict warning after submission: "⚠️ Compliance note: Firm actively trades this security"
- **Dashboard**: Display conflict details in compliance review view
- **Never auto-reject**: Always allow manager/compliance to approve despite conflicts

### FR-3: 3-Tier Lookup Completion

**FR-3.1: Implement Fallback Search**
- Complete track: `advanced_instrument_validation_20260115`
- Tier 1: `OracleBloomberg` (primary)
- Tier 2: `OracleMapInstSymbol` (exchange symbol fallback)
- Tier 3: `OracleProduct` (catch-all)
- Reference implementation: `/home/coder/repos/bodev/backoffice-web/compliance_portal/pages/pa_dealing/filters.py:182-226`

**FR-3.2: Integrate with Conflict Detection**
- After 3-tier lookup finds `inst_symbol`, query ProductUsage
- Use same short-circuit waterfall logic (stop at first tier with results)

## Technical Requirements

### TR-1: Database Models

**New Models** (in `src/pa_dealing/db/models/market.py`):

```python
class OracleProductUsage(Base):
    """Firm trading history synced from Oracle."""
    __tablename__ = "product_usage"
    __table_args__ = {'schema': 'bo_airflow'}

    id: Mapped[int] = mapped_column(BigInteger, primary_key=True)
    inst_symbol: Mapped[str] = mapped_column(String(30), index=True)
    inst_type: Mapped[str | None] = mapped_column(String(1))
    company: Mapped[str | None] = mapped_column(String(10))
    portfolio: Mapped[str | None] = mapped_column(String(30))
    last_trade_date: Mapped[datetime | None] = mapped_column(DateTime, index=True)
    last_position_date: Mapped[datetime | None] = mapped_column(DateTime)

class OraclePortfolioGroup(Base):
    """Portfolio/desk mapping synced from Oracle."""
    __tablename__ = "oracle_portfolio_group"
    __table_args__ = {'schema': 'bo_airflow'}

    pl_group_id: Mapped[int] = mapped_column(BigInteger, primary_key=True)
    portfolio: Mapped[str | None] = mapped_column(String(50))
    company: Mapped[str | None] = mapped_column(String(10))
    cost_centre: Mapped[str | None] = mapped_column(String(10))  # For desk name
    rp_portfolio: Mapped[str | None] = mapped_column(String(50))

class OracleHistoricPosition(Base):
    """Historic position data (mock for dev, real archive later)."""
    __tablename__ = "historic_position_mock"  # Mock table for dev

    id: Mapped[int] = mapped_column(BigInteger, primary_key=True)
    portfolio_id: Mapped[int | None] = mapped_column(Integer, index=True)
    inst_symbol: Mapped[str | None] = mapped_column(String(30), index=True)
    inst_type: Mapped[str | None] = mapped_column(String(1))
    settlement_date: Mapped[datetime | None] = mapped_column(DateTime, index=True)
    position_size: Mapped[int | None] = mapped_column(BigInteger)
    exchange_id: Mapped[int | None] = mapped_column(Integer)
```

### TR-2: Conflict Detection Service

**New Service** (in `src/pa_dealing/services/conflict_detection.py`):

```python
async def check_firm_trading_conflict(
    session: AsyncSession,
    inst_symbol: str
) -> ConflictResult:
    """
    Check if Mako actively trades this security.

    Reference: /home/coder/repos/bodev/backoffice-web/compliance_portal/pages/pa_dealing/rest.py:770-808
    """
    # Query ProductUsage for recent trading activity
    result = await session.execute(
        select(OracleProductUsage)
        .where(OracleProductUsage.inst_symbol == inst_symbol)
        .where(OracleProductUsage.last_trade_date >= date.today() - timedelta(days=90))
    )
    positions = result.scalars().all()

    if not positions:
        return ConflictResult(has_conflict=False)

    # Get desk names via PortfolioGroup
    desk_names = []
    for pos in positions:
        pg = await session.execute(
            select(OraclePortfolioGroup)
            .where(OraclePortfolioGroup.portfolio == pos.portfolio)
        )
        if pg_row := pg.scalar_one_or_none():
            desk_names.append(pg_row.rp_portfolio or pg_row.portfolio)

    return ConflictResult(
        has_conflict=True,
        conflict_type='firm_trading',
        portfolios=[p.portfolio for p in positions],
        desk_names=list(set(desk_names)),
        details=f"Firm actively trades across {len(positions)} portfolios: {', '.join(desk_names[:3])}"
    )
```

### TR-3: Risk Engine Integration

**Update**: `src/pa_dealing/services/risk.py`

Add conflict check to existing risk assessment:

```python
async def assess_risk(session: AsyncSession, request: PADRequest) -> RiskAssessment:
    """Enhanced with firm trading conflict detection."""

    # Existing checks (restricted list, risk scoring)
    assessment = await _base_risk_assessment(session, request)

    # NEW: Firm trading conflict check
    conflict = await check_firm_trading_conflict(session, request.security.inst_symbol)

    if conflict.has_conflict:
        request.has_conflict = True
        request.conflict_comments = conflict.details
        # Optional: Increase risk score for conflicts
        assessment.risk_score += 10

    return assessment
```

## Reference Code (Legacy PA Dealing System)

### Primary References

**Conflict Detection Query**:
- File: `/home/coder/repos/bodev/backoffice-web/compliance_portal/pages/pa_dealing/rest.py`
- Lines: 770-808
- Logic: ProductUsage query with HistoricPositionVw subquery for position_size and desk_name annotation

**3-Tier Instrument Lookup**:
- File: `/home/coder/repos/bodev/backoffice-web/compliance_portal/pages/pa_dealing/filters.py`
- Lines: 182-226
- Logic: Short-circuit waterfall (Bloomberg → MapInstSymbol → Product)

**ProductUsage Serializer**:
- File: `/home/coder/repos/bodev/backoffice-web/compliance_portal/pages/pa_dealing/serializers.py`
- Lines: 542-600
- Fields: desk_name, buysell_display, description (via Product subquery)

### Schema References

**ProductUsage Model**:
- File: `/home/coder/repos/bodev/backoffice-web/database_models/models/tables/table_models.py`
- Lines: 5448-5461

**HistoricPositionVw Model**:
- File: `/home/coder/repos/bodev/backoffice-web/database_models/models/views/views_models.py`
- Lines: 695-754
- Note: We only need 6 fields (portfolio_id, inst_symbol, inst_type, settlement_date, position_size, exchange_id)

**OraclePortfolioGroup Model**:
- File: `/home/coder/repos/bodev/bodev_backend/backoffice-db-models/backoffice_db_models/models.py`
- Lines: 4795-4823

**EmployeeDetailVw Fields Used**:
- File: `/home/coder/repos/bodev/backoffice-web/compliance_portal/pages/pa_dealing/serializers.py`
- Lines: 38-41, 115-120
- Fields accessed: `id`, `forename`, `surname`, `manager_id`, `division_id`
- Note: We already have these in `oracle_employee` - no need for view

## Out of Scope

**Explicitly NOT included in this track**:

1. **Automatic Trade Rejection**: System will NEVER auto-reject based on conflicts (advisory only)
2. **Real-Time Archive Database Connection**: Separate track for direct Oracle archive queries
3. **Complete HistoricPosition Migration**: Use mock sample data (1 week), full migration is separate effort
4. **Legacy Field Refactoring**: Track `legacy_field_review_20260120` handles is_derivative, is_leveraged integration
5. **Employee Enhanced View**: We have needed fields in `oracle_employee`, skip EmployeeDetailVw
6. **Advanced Conflict Types**: Focus on firm trading conflicts only (no research analyst coverage, M&A deal conflicts, etc.)

## Success Criteria

1. ✅ ProductUsage table synced to dev database (`bo_airflow.product_usage`)
2. ✅ OraclePortfolioGroup SQLAlchemy model created and working
3. ✅ HistoricPosition mock schema created with 1 week sample data
4. ✅ 3-tier instrument lookup fully implemented (Advanced Instrument Validation track complete)
5. ✅ Conflict detection service returns firm trading conflicts
6. ✅ Risk engine enriches PADRequest with `has_conflict` and `conflict_comments`
7. ✅ Slack bot displays conflict warnings (advisory only)
8. ✅ Dashboard compliance view shows conflict details
9. ✅ Integration tests verify conflict detection logic
10. ✅ E2E test: Submit request for security Mako trades → conflict flagged → compliance approves despite conflict

## Acceptance Criteria

**Given** an employee submits a personal trading request for a security
**When** Mako's firm actively trades that security (ProductUsage.last_trade_date < 90 days)
**Then** the system:
- Sets `PADRequest.has_conflict = True`
- Populates `conflict_comments` with portfolio and desk details
- Displays warning in Slack: "⚠️ Compliance note: Firm actively trades this security"
- Shows conflict details in dashboard compliance view
- **Still routes to manager/compliance for approval** (no auto-rejection)

**Given** an employee submits a request for a security Mako doesn't trade
**When** ProductUsage has no recent records (>90 days or none)
**Then** `has_conflict = False` and request proceeds normally

## Non-Functional Requirements

- **Performance**: Conflict check must complete within 2 seconds
- **Data Freshness**: ProductUsage synced daily (acceptable 24hr lag)
- **Audit Trail**: Log all conflict detections in `audit_log` table
- **Extensibility**: Design for future conflict types (research coverage, M&A deals)

## Dependencies

**Blocking**:
- ✅ Multi-Environment Database Migration track (`db_migration_20251230`) - Need dev database access
- ⏳ Advanced Instrument Validation track (`advanced_instrument_validation_20260115`) - Need 3-tier lookup

**Non-Blocking**:
- Legacy Field Review track (`legacy_field_review_20260120`) - Can implement in parallel

## Future Enhancements (Separate Tracks)

1. **Real-Time Archive Connection**: Direct query to Oracle archive for HistoricPosition
2. **Advanced Conflict Types**: Research analyst coverage, M&A deal conflicts, client relationship conflicts
3. **Conflict Resolution Workflow**: Compliance can add conflict mitigation notes, set trade limits
4. **Historical Conflict Analysis**: Dashboard showing conflict trends, most-conflicted securities
