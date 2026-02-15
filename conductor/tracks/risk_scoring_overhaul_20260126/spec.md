# Risk Scoring Overhaul & Oracle Position Enrichment

## Overview

Overhaul the risk scoring system to align with spec.md requirements and regulatory policy. Remove instant-rejection criteria from point-based scoring (they become "Strongly Advise to Reject" advisories). Add Oracle DB integration for Mako position enrichment (trade direction, position value, materiality ratio from HISTORIC_POSITION_VW). Make all scoring factors configurable via Dashboard UI.

## Background

The current scoring system includes criteria that should trigger instant rejection per policy (derivatives, leveraged trades, restricted list, etc.). These are currently mixed into the point-based risk calculation, diluting their severity.

Per policy, these should be **mandatory blocks** (future) or **strong advisories** (Phase 1). The scoring system should focus on risk classification (LOW/MEDIUM/HIGH) for routing decisions, not rejection logic.

## Functional Requirements

### FR0: Dashboard Approval Slack Notifications (Prerequisite)

**Problem:** Dashboard approvals only updated the database - they did not trigger Slack notifications like the Slack button flow did.

**Solution:** Added `_send_next_notification()` call to both `approve_request()` and `decline_request()` API endpoints in `routes/requests.py`.

**Status:** Implementation complete, tests created, pending verification.

### FR1: Oracle DB Integration for Position Enrichment

**Connection:**
- Server: `uk01vdb007.uk.makoglobal.com`
- SID: `dev`
- Credentials: `pad_app_user` / `padd_app_pass`
- Driver: `oracledb` with SQLAlchemy async

**Workflow:**
1. Search `ProductUsage` to find Mako activity for the instrument
2. Get `last_position_date` from ProductUsage result
3. Query `HISTORIC_POSITION_VW` using date + symbol + portfolio
4. Determine trade direction from `position_size`:
   - `> 0` → Mako is **Long** (Buy)
   - `< 0` → Mako is **Short** (Sell)
   - `== 0` or NULL → Mako is **Inactive**
5. Capture Mako position size (absolute value of `position_size`)
6. Calculate materiality ratio: `employee_value / mako_position_value`

**Query HISTORIC_POSITION_VW for:**
- `position_size` (for direction: >0=Long, <0=Short)
- `position_value_gbp` (absolute value, for materiality analysis)
- `last_update_date`

**Derived Fields:**
- Trade direction: `LONG` | `SHORT` | `INACTIVE`
- Mako position value: £X,XXX,XXX
- Materiality ratio: employee_value / mako_value (for SMF16 context)

**Model:**
- Create `OracleHistoricPositionVw` SQLAlchemy model for the view

### FR2: Simplified Risk Scoring

Remove instant-rejection criteria from scoring. Remove "SMF16 required" as input (circular - HIGH risk routes to SMF16).

**Risk Factors:**

| Factor | LOW | MEDIUM | HIGH |
|--------|-----|--------|------|
| Instrument type | Standard equity | ETF, Bond, Complex | - |
| Mako traded | >3 months / never | - | Within 3 months OR Active |
| Direction match | N/A (no Mako activity) | Same direction | Opposite direction |
| Employee role | Standard | Manager (configurable) | Trading desk / Senior (configurable) |
| Employee position size | <£100k | £100k - £1M | >£1M |
| Connected person | No | - | Yes |

**Position Size Clarification:**
- "Employee position size" = employee's declared trade value in GBP (from PAD request form)
- Mako position size is captured separately for SMF16 review context (see FR1)

**Direction Match Logic:**
- Mako active + **OPPOSITE** direction → Automatic **HIGH** (severe conflict / front-running)
- Mako active + **SAME** direction → **HIGH** (potential insider trading concern)
- Mako historical only (no active position) → Use time-based factor only

**Mako Traded Logic:**
- Binary escalation: If Mako traded within 3 months → Always **HIGH** (escalate to SMF16)
- >3 months or never → **LOW**

**Scoring Logic:**
- Any HIGH factor → **HIGH** risk (route to SMF16)
- 2+ MEDIUM factors → **MEDIUM** risk (route to Compliance)
- Otherwise → **LOW** risk (auto-approve with logging)

### FR3: "Strongly Advise to Reject" Advisory System

**Criteria that trigger advisory:**
1. Prohibited Instruments: Derivatives, Leveraged, Spread bets, Exchange-traded futures/options
2. Restricted List: Instrument on internal restricted list
3. Inside Information: Requester did NOT confirm absence
4. Holding Period: 30-day rule violated or reset logic violated
5. Desk Match: Employee's desk has active position (insider dealing risk)
6. Missing Data: No instrument code (ISIN/ticker/Bloomberg)

**Advisory vs Block (future-proof):**
- Phase 1: All criteria trigger "Strongly Advise to Reject" warning
- Future: Configurable toggle per criteria for "Strongly Advise" vs "Auto Block"

### FR4: Dashboard Warning UI

**Design:**
- Light red background (`#fef2f2`), subtle red border (`#fca5a5`)
- 4px border-radius, 12px/16px padding, 13px font
- Header: 🚫 icon + "AI Strongly Advises Rejection" (bold, `#991b1b`)
- Summary: "Critical issues detected: [Issue1], [Issue2], ..."
- Link: "View compliance details →" (`#dc2626`, underlined)
- Placement: Top of Compliance Analysis section, above Risk Score
- Behavior: Dismissable/expandable, only shows when triggered

### FR5: Slack Warning UI

**Block Kit Structure:**
```json
{
  "type": "header",
  "text": {
    "type": "plain_text",
    "text": "🚫 AI Strongly Advises Rejection",
    "emoji": true
  }
}
```

Followed by context block listing triggered criteria:
- Appears in both Compliance channel notifications and Manager DM alerts
- Lists specific issues (e.g., "Desk Match (insider dealing)", "Derivative instrument")
- Includes Mako position context: direction, value, materiality ratio

### FR6: Dashboard Config - Auto-Reject Criteria Tab

**New tab** in existing config section (alongside Holding Period, Broker Notes Period):

**Sections:**
1. **Position Size Thresholds**
   - LOW threshold (default: £100,000)
   - HIGH threshold (default: £1,000,000)

2. **Employee Risk Categories**
   - HIGH risk employees (multi-select list)
   - MEDIUM risk employees (multi-select list)
   - Default: Standard (everyone else)

3. **Auto-Reject Criteria**
   - Each criterion with toggle (enabled/disabled) and mode dropdown:
     - Derivatives: ☑️ [Strongly Advise ▼]
     - Leveraged: ☑️ [Strongly Advise ▼]
     - Spread Bets: ☑️ [Strongly Advise ▼]
     - Restricted List: ☑️ [Strongly Advise ▼]
     - Inside Info Not Confirmed: ☑️ [Strongly Advise ▼]
     - Holding Period Violation: ☑️ [Strongly Advise ▼]
     - Desk Match: ☑️ [Strongly Advise ▼]
   - Mode options: "Strongly Advise" | "Auto Block" (future)

4. **Mako Trading Lookback**
   - Period in months (default: 3)

## Non-Functional Requirements

- NFR1: Oracle queries must complete within 2 seconds
- NFR2: Config changes take effect immediately (no restart required)
- NFR3: All advisory triggers logged for audit trail
- NFR4: Backward compatible - existing requests continue to work

## Acceptance Criteria

- [ ] AC1: Oracle connection to uk01vdb007 established and tested
- [ ] AC2: HISTORIC_POSITION_VW model queries successfully return position data
- [ ] AC3: Trade direction (Long/Short/Inactive) correctly derived from position_size
- [ ] AC4: Mako position value and materiality ratio calculated correctly
- [ ] AC5: Risk scoring uses only the 6 simplified factors (no circular SMF16 logic)
- [ ] AC6: Mako traded within 3 months always escalates to HIGH
- [ ] AC7: Direction match (opposite) triggers HIGH risk
- [ ] AC8: Advisory warnings appear in Dashboard for all 6 criteria
- [ ] AC9: Advisory warnings appear in Slack (Compliance + Manager) for all 6 criteria
- [ ] AC10: Config UI allows modification of all thresholds and criteria
- [ ] AC11: Config changes persist and apply immediately
- [ ] AC12: Existing tests pass (backward compatibility)
- [ ] AC13: New unit tests for Oracle integration, scoring, and advisory logic

## Out of Scope

- Spread betting attestation question (separate track)
- Actual auto-rejection/blocking (Phase 2 - this track is advisory only)
- MAR/MiFID breach inference (requires legal review)
- Migration of historical scoring data
