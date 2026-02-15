# Risk Scoring System Change Documentation

## Overview

This document describes the changes from the old point-based risk scoring system to the new simplified 6-factor risk scoring system.

## Old System (Point-Based, 0-100)

The old system used a point-based approach where various factors contributed points to a cumulative score:

### Factors and Point Values (OLD)

| Factor | Points | Threshold |
|--------|--------|-----------|
| Derivative product | +30 | - |
| Leveraged product | +30 | - |
| Cryptocurrency | +25 | - |
| Prohibited product | 100 (max) | - |
| Restricted security | 100 (max) | - |
| Holding period violation | +40 | - |
| High value trade | +20 | >$50,000 |
| Medium value trade | +10 | >$10,000 |
| High conflict (Mako) | +40 | <7 days |
| Medium conflict (Mako) | +15 | 7-30 days |
| Low conflict (Mako) | +5 | 30-90 days |
| Excessive trading | +15 | >10 requests/30 days |
| Related party | +15 | - |
| Investment staff | +15 | - |
| Small-cap/illiquid | +10 | - |
| Significant position increase | +10 | trade > existing position |
| MAR HIGH severity | +40 | - |
| MAR MEDIUM severity | +20 | - |

### Risk Level Thresholds (OLD)

- **HIGH**: Score >= 40
- **MEDIUM**: Score >= 10
- **LOW**: Score < 10

### Problems with Old System

1. **Mixed concerns**: Instant-rejection criteria (derivatives, restricted list) were scored alongside risk factors
2. **Circular logic**: "SMF16 required" was used as an input to scoring, but scoring determined SMF16 routing
3. **Complex thresholds**: Multiple overlapping time-based thresholds for conflicts
4. **Hard to explain**: Point-based scores don't clearly communicate why a request is risky

---

## New System (6-Factor, Level-Based)

The new system uses 6 distinct factors, each rated as LOW/MEDIUM/HIGH, with simple aggregation rules.

### The 6 Factors (NEW)

| # | Factor | LOW | MEDIUM | HIGH |
|---|--------|-----|--------|------|
| 1 | **Instrument Type** | Standard equity | ETF, Bond, Fund, Complex | - |
| 2 | **Mako Traded** | >3 months / never | - | Within 3 months OR active position |
| 3 | **Direction Match** | N/A (no Mako activity) | Same direction | Opposite direction |
| 4 | **Employee Role** | Standard employee | Manager, Analyst, Director | Trading desk, PM, Senior Trader |
| 5 | **Employee Position Size** | <£100k | £100k - £1M | >£1M |
| 6 | **Connected Person** | No | - | Yes |

### Aggregation Rules (NEW)

- **Any HIGH** factor → Overall **HIGH** risk
- **2+ MEDIUM** factors → Overall **MEDIUM** risk
- Otherwise → Overall **LOW** risk

### Routing (NEW)

- **HIGH** → SMF16 escalation required
- **MEDIUM** → Compliance review
- **LOW** → Auto-approve eligible

---

## Factors REMOVED from Scoring

The following factors are **no longer part of risk scoring**. They are now handled by the **Advisory System** (Phase 3) as potential auto-reject criteria:

### 1. Derivative Products
- **Old**: +30 points
- **New**: Handled by advisory system as "Strongly Advise to Reject"
- **Rationale**: Per policy, derivatives are prohibited - they shouldn't inflate a score, they should trigger an advisory

### 2. Leveraged Products
- **Old**: +30 points
- **New**: Handled by advisory system
- **Rationale**: Same as derivatives - prohibited per policy

### 3. Cryptocurrency
- **Old**: +25 points
- **New**: Handled by advisory system
- **Rationale**: Prohibited product type

### 4. Prohibited Product (instant max)
- **Old**: 100 points (automatic HIGH)
- **New**: Handled by advisory system
- **Rationale**: Should trigger advisory warning, not just a high score

### 5. Restricted Security (instant max)
- **Old**: 100 points (automatic HIGH)
- **New**: Handled by advisory system
- **Rationale**: Restricted list should be an advisory, not scoring

### 6. Holding Period Violation
- **Old**: +40 points
- **New**: Handled by advisory system
- **Rationale**: Policy violation = advisory, not scoring factor

### 7. Excessive Trading Pattern
- **Old**: +15 points (>10 requests in 30 days)
- **New**: REMOVED (monitoring only)
- **Rationale**: This is a pattern detection issue, not a per-request risk factor

### 8. Small-cap/Illiquid Securities
- **Old**: +10 points
- **New**: REMOVED
- **Rationale**: Not a strong risk indicator per current policy

### 9. Significant Position Increase
- **Old**: +10 points (if trade > existing position)
- **New**: REMOVED
- **Rationale**: Direction Match factor captures the relevant conflict concern

### 10. MAR Severity Flags
- **Old**: +20/+40 points
- **New**: REMOVED from scoring (advisory system)
- **Rationale**: MAR concerns are serious compliance issues, not scoring factors

---

## Factors ADDED to Scoring

### Direction Match (NEW)

This factor uses Oracle HISTORIC_POSITION_VW to determine Mako's trade direction:

- **N/A**: No Mako activity for this security
- **MEDIUM**: Employee trading same direction as Mako (both buying or both selling)
- **HIGH**: Employee trading **opposite** direction to Mako (potential front-running)

This is derived from `position_size` in HISTORIC_POSITION_VW:
- `position_size > 0` → Mako is LONG
- `position_size < 0` → Mako is SHORT
- `position_size = 0` → INACTIVE

---

## Factors MODIFIED

### Mako Trading Activity

**Old**: Three-tier time-based system
- High conflict: <7 days (+40)
- Medium conflict: 7-30 days (+15)
- Low conflict: 30-90 days (+5)

**New**: Binary system
- HIGH: Within 3 months OR has active position
- LOW: >3 months OR never traded

**Rationale**: Simplified to a single lookback period. If Mako traded recently, escalate to SMF16.

### Employee Role

**Old**: Binary (is_investment_staff: +15)

**New**: Three-tier based on configurable role lists
- LOW: Standard employee
- MEDIUM: Manager, Analyst, Director
- HIGH: Trading desk, Portfolio Manager, Senior Trader

**Rationale**: More granular classification with configurable role lists.

### Trade Value

**Old**: Two thresholds (USD-based)
- High: >$50,000 (+20)
- Medium: >$10,000 (+10)

**New**: Three tiers (GBP-based)
- LOW: <£100,000
- MEDIUM: £100,000 - £1,000,000
- HIGH: >£1,000,000

**Rationale**: Aligned with regulatory thresholds and simplified to factor levels.

---

## Configuration

The new system supports configuration of:

- Position size thresholds (£100k, £1M defaults)
- Mako lookback period (3 months default)
- High-risk employee role list
- Medium-risk employee role list
- Medium-risk instrument types

Configuration is stored in `RiskScoringConfig` and can be loaded from the database.

---

## Migration Notes

1. **Backward Compatibility**: The old `RiskClassifier` class remains available for reference
2. **New Module**: New scoring in `src/pa_dealing/agents/orchestrator/risk_scoring.py`
3. **Tests**: New tests in `tests/unit/test_risk_scoring.py`
4. **Advisory System**: Removed factors will be implemented in Phase 3 as advisory warnings

---

## Summary

| Aspect | Old System | New System |
|--------|------------|------------|
| Approach | Point-based (0-100) | Factor-based (LOW/MEDIUM/HIGH) |
| # of factors | 15+ | 6 |
| Aggregation | Sum points, threshold | Any HIGH = HIGH, 2+ MEDIUM = MEDIUM |
| Prohibited items | Scored (100 pts) | Advisory warnings (separate) |
| Direction matching | None | Yes (from Oracle position data) |
| Configurability | 20+ config values | 5 key settings |
