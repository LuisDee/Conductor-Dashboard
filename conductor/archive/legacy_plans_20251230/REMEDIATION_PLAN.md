# PA Dealing System - Comprehensive Remediation Plan

## Document Control
- **Version:** 1.2
- **Date:** 2025-12-18
- **Status:** IN PROGRESS - Phase 1, 2, 3, 4 & 5 Complete, Phase 6 (Dashboard) Remaining

---

## Executive Summary

This plan addresses all gaps identified between the current implementation and the PA Dealing specification requirements. Key decisions:

1. **Slack is the primary UI** - All employee interactions via Slack chatbot
2. **Schema can be redesigned** - No requirement to keep legacy PERSONAL_ACCOUNT_DEALING structure
3. **Migration required** - All historical records must be migrated to new schema
4. **No Boffin integration** - Standalone system with optional compliance dashboard

---

## Current State Assessment

### What's Working
- Slack chatbot for PAD request submission
- Security lookup from Bloomberg reference data
- Employee identification from Slack profile
- Manager/Compliance approval workflow via Slack buttons
- Basic risk classification (LOW/MEDIUM/HIGH)
- Holding period and conflict checks (partial)
- Thread-based conversations with status reactions

### Critical Gaps
1. **Data Integrity:** Instrument identifiers not properly stored
2. **Policy Rules:** Check results not persisted, exemptions not implemented
3. **Auto-Approval:** Low-risk requests still require manual approval
4. **Post-Trade Monitoring:** No execution tracking or broker reconciliation
5. **Slack Features:** Users can't query positions, trade history, or status
6. **Audit Trail:** Incomplete logging of AI decisions and state changes

---

## Phase 1: Schema Redesign & Data Migration

### 1.1 New Schema Design

Replace the legacy `personal_account_dealing` table with a normalized structure:

#### Core Tables

```sql
-- Main PAD request table (simplified, normalized)
CREATE TABLE pad_request (
    id SERIAL PRIMARY KEY,

    -- Request metadata
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Requester
    employee_id INTEGER NOT NULL REFERENCES employee(id),

    -- Security (foreign key to normalized security table)
    security_id INTEGER NOT NULL REFERENCES security(id),

    -- Trade details
    direction VARCHAR(4) NOT NULL CHECK (direction IN ('BUY', 'SELL')),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    estimated_value DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',

    -- Connected person
    is_connected_person BOOLEAN NOT NULL DEFAULT FALSE,
    connected_person_relation VARCHAR(100),

    -- Declarations
    has_existing_position BOOLEAN NOT NULL DEFAULT FALSE,
    existing_position_quantity INTEGER DEFAULT 0,
    justification TEXT,

    -- AI Assessment (stored JSON)
    ai_risk_assessment JSONB,

    -- Status tracking
    status VARCHAR(20) NOT NULL DEFAULT 'pending_manager',
    -- pending_manager, pending_compliance, pending_smf16, approved, declined, expired, executed

    -- Slack tracking
    slack_channel_id VARCHAR(20),
    slack_thread_ts VARCHAR(20),
    slack_user_id VARCHAR(20)
);

-- Normalized security table (populated from Bloomberg lookup)
CREATE TABLE security (
    id SERIAL PRIMARY KEY,
    bloomberg VARCHAR(30) NOT NULL,  -- "NVDA US" (without Equity suffix)
    ticker VARCHAR(20),              -- "NVDA"
    isin VARCHAR(12),
    sedol VARCHAR(7),
    cusip VARCHAR(9),
    description VARCHAR(200) NOT NULL,
    security_type VARCHAR(20),       -- equity, etf, bond, derivative, etc.
    currency VARCHAR(3),
    exchange VARCHAR(10),
    sector VARCHAR(50),
    industry VARCHAR(50),
    is_prohibited BOOLEAN DEFAULT FALSE,
    prohibition_reason VARCHAR(200),
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(bloomberg)
);

-- Approval workflow table (one row per approval action)
CREATE TABLE pad_approval (
    id SERIAL PRIMARY KEY,
    request_id INTEGER NOT NULL REFERENCES pad_request(id),
    approval_type VARCHAR(20) NOT NULL,  -- manager, compliance, smf16
    approver_id INTEGER NOT NULL REFERENCES employee(id),
    decision VARCHAR(10) NOT NULL,       -- approved, declined
    comments TEXT,
    decided_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Compliance-specific fields
    restricted_check BOOLEAN,
    holding_period_check BOOLEAN,
    conflict_check BOOLEAN,
    conflict_details TEXT
);

-- Execution tracking table
CREATE TABLE pad_execution (
    id SERIAL PRIMARY KEY,
    request_id INTEGER NOT NULL REFERENCES pad_request(id),
    executed_at TIMESTAMP,
    execution_price DECIMAL(15,4),
    execution_quantity INTEGER,
    broker_reference VARCHAR(50),
    contract_note_received BOOLEAN DEFAULT FALSE,
    contract_note_date TIMESTAMP,
    variance_flag BOOLEAN DEFAULT FALSE,  -- True if >5% variance from estimate
    variance_details TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Breach/alert tracking
CREATE TABLE pad_breach (
    id SERIAL PRIMARY KEY,
    request_id INTEGER REFERENCES pad_request(id),
    breach_type VARCHAR(50) NOT NULL,
    -- execution_overdue, no_contract_note, price_variance, volume_variance,
    -- traded_without_approval, holding_period_violation
    detected_at TIMESTAMP NOT NULL DEFAULT NOW(),
    details TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    resolved_at TIMESTAMP,
    resolved_by INTEGER REFERENCES employee(id),
    resolution_notes TEXT
);

-- AI decision audit log
CREATE TABLE pad_ai_decision (
    id SERIAL PRIMARY KEY,
    request_id INTEGER NOT NULL REFERENCES pad_request(id),
    decision_type VARCHAR(30) NOT NULL,  -- risk_assessment, auto_approve, recommendation
    model_version VARCHAR(50),
    prompt_summary TEXT,
    response JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Employee position tracking (calculated from approved requests)
CREATE VIEW employee_positions AS
SELECT
    pr.employee_id,
    s.bloomberg,
    s.description,
    SUM(CASE WHEN pr.direction = 'BUY' THEN pr.quantity ELSE -pr.quantity END) as net_position,
    MAX(pr.updated_at) as last_trade_date
FROM pad_request pr
JOIN security s ON s.id = pr.security_id
WHERE pr.status = 'executed'
GROUP BY pr.employee_id, s.bloomberg, s.description
HAVING SUM(CASE WHEN pr.direction = 'BUY' THEN pr.quantity ELSE -pr.quantity END) != 0;
```

### 1.2 Migration Strategy

1. Create new schema alongside existing
2. Write migration script to:
   - Normalize securities into `security` table
   - Map old records to new `pad_request` structure
   - Reconstruct approval history into `pad_approval`
   - Mark historical records appropriately
3. Validate migration with checksums
4. Switch application to new schema
5. Keep old table read-only for audit purposes

### 1.3 Deliverables
- [x] New schema SQL scripts (models.py: PADRequest, PADApproval, PADExecution, Security, PADBreach, PADAIDecision)
- [x] Migration script with validation (scripts/migrate_oracle_data.py)
- [x] Rollback procedure (DROP CASCADE on new tables)
- [x] Data validation report (scripts/validate_schema.py)

---

## Phase 2: Policy Rule Engine (FR2)

### 2.1 Prohibited Instruments

Create comprehensive prohibited product detection:

```python
PROHIBITED_TYPES = {
    'derivative': 'Derivatives are prohibited under PA Dealing policy',
    'cfd': 'CFDs are prohibited under PA Dealing policy',
    'spread_bet': 'Spread betting is prohibited under PA Dealing policy',
    'leveraged_etf': 'Leveraged ETFs are prohibited under PA Dealing policy',
    'futures': 'Exchange traded futures are prohibited',
    'options': 'Exchange traded options are prohibited',
}

PROHIBITED_DETECTION_RULES = [
    # Bloomberg yellow key indicators
    {'field': 'yellow_key', 'values': ['Comdty', 'Index'], 'type': 'derivative'},
    # Description keywords
    {'field': 'description', 'contains': ['leveraged', '2x', '3x', '-2x', '-3x'], 'type': 'leveraged_etf'},
    {'field': 'description', 'contains': ['spread bet', 'cfd'], 'type': 'cfd'},
    # Instrument type
    {'field': 'inst_type', 'values': ['F', 'O'], 'type': 'futures_options'},
]
```

### 2.2 Exemptions Engine

```python
EXEMPTIONS = {
    'discretionary_managed': {
        'description': 'Discretionary managed accounts are exempt from pre-approval',
        'requires_declaration': True,
        'auto_approve': True,
    },
    'index_fund': {
        'description': 'Passive index funds exempt from conflict checks',
        'skip_conflict_check': True,
    },
    'spot_fx_non_speculative': {
        'description': 'Spot FX for non-speculative purposes (e.g., holiday money)',
        'max_value': 10000,
        'requires_justification': True,
    },
}
```

### 2.3 Enhanced Holding Period Logic

```python
def check_holding_period(employee_id: int, security_id: int, direction: str) -> HoldingPeriodResult:
    """
    30-day holding period with reset logic:
    - Cannot sell within 30 days of buy
    - Additional buys reset the 30-day clock
    - Returns days remaining and any violations
    """
    # Get all approved trades for this employee/security
    trades = get_employee_trades(employee_id, security_id)

    if direction == 'SELL':
        # Find most recent BUY
        last_buy = get_last_buy(trades)
        if last_buy:
            days_held = (now() - last_buy.executed_at).days
            if days_held < 30:
                return HoldingPeriodResult(
                    can_trade=False,
                    days_remaining=30 - days_held,
                    message=f"Holding period violation: {30-days_held} days remaining"
                )

    elif direction == 'BUY':
        # Warn that this will reset holding period if they have existing position
        existing = get_net_position(employee_id, security_id)
        if existing > 0:
            return HoldingPeriodResult(
                can_trade=True,
                warning="This purchase will reset your 30-day holding period"
            )

    return HoldingPeriodResult(can_trade=True)
```

### 2.4 Deliverables
- [x] Prohibited instrument detection module (policy_engine.py: ProhibitedInstrumentDetector)
- [x] Exemptions configuration and engine (policy_engine.py: ExemptionsEngine)
- [x] Enhanced holding period with reset logic (policy_engine.py: EnhancedHoldingPeriodChecker)
- [x] Policy check results stored on PAD record (schemas.py: ai_risk_assessment JSONB)
- [x] Unit tests for all policy rules (test_orchestrator.py: 28 new tests)

---

## Phase 3: AI Risk & Recommendation Engine (FR3, FR4)

### 3.1 Risk Classification Model

```python
@dataclass
class RiskAssessment:
    classification: str  # LOW, MEDIUM, HIGH
    score: int  # 0-100
    recommendation: str  # APPROVE, REVIEW, REJECT, ESCALATE_SMF16
    policy_flags: list[str]
    explanation: str
    suggested_approver: str  # COMPLIANCE, SMF16
    auto_approve_eligible: bool

def assess_risk(request: PADRequest) -> RiskAssessment:
    score = 0
    flags = []

    # Factor 1: Mako trading activity (spec: "must escalate if traded in last 3 months")
    mako_activity = check_mako_positions(request.security_id)
    if mako_activity.traded_recently:
        score += 40
        flags.append(f"Security traded by Mako {mako_activity.days_ago} days ago")

    # Factor 2: Trade value
    if request.estimated_value > 50000:
        score += 20
        flags.append("High value trade (>$50,000)")
    elif request.estimated_value > 10000:
        score += 10
        flags.append("Medium value trade (>$10,000)")

    # Factor 3: Connected person
    if request.is_connected_person:
        score += 15
        flags.append(f"Connected person: {request.connected_person_relation}")

    # Factor 4: Employee role
    if employee_is_investment_staff(request.employee_id):
        score += 15
        flags.append("Employee has investment decision-making role")

    # Factor 5: Position size relative to existing
    if request.has_existing_position:
        if request.quantity > request.existing_position_quantity:
            score += 10
            flags.append("Significant position increase")

    # Factor 6: Security type
    if is_small_cap(request.security_id):
        score += 10
        flags.append("Small-cap/illiquid security")

    # Determine classification
    if score >= 50:
        classification = "HIGH"
        recommendation = "ESCALATE_SMF16" if score >= 70 else "REVIEW"
        suggested_approver = "SMF16" if score >= 70 else "COMPLIANCE"
        auto_approve = False
    elif score >= 25:
        classification = "MEDIUM"
        recommendation = "REVIEW"
        suggested_approver = "COMPLIANCE"
        auto_approve = False
    else:
        classification = "LOW"
        recommendation = "APPROVE"
        suggested_approver = "COMPLIANCE"
        auto_approve = True

    return RiskAssessment(
        classification=classification,
        score=score,
        recommendation=recommendation,
        policy_flags=flags,
        explanation=generate_explanation(flags, classification),
        suggested_approver=suggested_approver,
        auto_approve_eligible=auto_approve
    )
```

### 3.2 Auto-Approve Logic

```python
async def process_new_request(request: PADRequest) -> ProcessingResult:
    # Run all compliance checks
    checks = await run_compliance_checks(request)

    if not checks.all_passed:
        return ProcessingResult(
            action="BLOCKED",
            reason=checks.first_failure_reason
        )

    # Get AI risk assessment
    risk = assess_risk(request)

    # Store AI decision for audit
    await log_ai_decision(request.id, "risk_assessment", risk)

    # Auto-approve if eligible
    if risk.auto_approve_eligible and settings.auto_approve_enabled:
        await auto_approve_request(request, risk)
        return ProcessingResult(
            action="AUTO_APPROVED",
            risk_assessment=risk
        )

    # Route to appropriate approver
    if risk.suggested_approver == "SMF16":
        await send_smf16_notification(request, risk)
    else:
        await send_manager_notification(request, risk)

    return ProcessingResult(
        action="PENDING_APPROVAL",
        risk_assessment=risk
    )
```

### 3.3 Deliverables
- [x] Risk scoring algorithm with all factors (risk_classifier.py: Mako +40, value +20/10, connected +15, investment staff +15, small-cap +10, position increase +10)
- [x] Auto-approve flow with full audit trail (compliance_decision_service.py: execute_auto_approve, log_decision)
- [x] SMF16 escalation workflow (compliance_decision_service.py: route_to_smf16)
- [x] Decision rationale generation (compliance_decision_service.py: generate_rationale)
- [x] Configuration for risk thresholds (config.py: RiskThresholds with DB loading)

---

## Phase 4: Post-Trade Monitoring (FR6)

### 4.1 Execution Confirmation Flow

**Slack interaction after approval:**

```
Bot: Your PAD request #123 has been approved!

*Next Steps:*
1. Execute your trade within 2 business days
2. Reply here with execution details: `executed <price> <quantity>`
3. Upload your contract note when received

_Deadline: December 20, 2025_
```

**Employee commands:**
- `executed 150.25 100` - Record execution at $150.25 for 100 shares
- `upload contract note` - Trigger file upload flow
- `extend deadline` - Request extension (requires justification)

### 4.2 Monitoring Jobs

```python
# Scheduled jobs (run daily)

async def check_execution_deadlines():
    """Flag requests not executed within 2 business days"""
    overdue = await get_overdue_requests()
    for request in overdue:
        await create_breach(
            request_id=request.id,
            breach_type='execution_overdue',
            details=f"Approved {request.approved_at}, deadline was {request.deadline}"
        )
        await send_reminder_to_employee(request)
        await notify_compliance(request, "Execution overdue")

async def check_contract_notes():
    """Flag executed trades without contract notes after 30 days"""
    missing = await get_executed_without_contract_note(days=30)
    for request in missing:
        await create_breach(
            request_id=request.id,
            breach_type='no_contract_note',
            details="Contract note not received within 30 days"
        )
        await notify_compliance(request, "Contract note missing")

async def check_holding_period_expiries():
    """Notify employees of upcoming holding period expiries"""
    expiring = await get_holding_periods_expiring_soon(days=7)
    for position in expiring:
        await send_holding_period_reminder(position)

async def check_new_mako_conflicts():
    """Alert if Mako starts trading a security an employee holds"""
    employee_positions = await get_all_employee_positions()
    for position in employee_positions:
        if await mako_started_trading_recently(position.security_id):
            await notify_compliance(
                f"New conflict: {position.employee_name} holds {position.security} "
                f"which Mako started trading"
            )
```

### 4.3 Deliverables
- [x] Execution confirmation Slack flow (chatbot.py: record_execution tool, tools.py: record_execution function)
- [x] Contract note tracking (monitoring/jobs.py: check_contract_notes - creates breaches for missing notes after 30 days)
- [x] Deadline tracking and reminders (monitoring/jobs.py: check_execution_deadlines - 2 business day deadline)
- [x] Breach detection scheduled jobs (monitoring/jobs.py: all 4 jobs create PADBreach records)
- [x] Compliance alert system (monitoring/jobs.py: Slack alerts to compliance channel and employee DMs)
- [x] Holding period expiry notifications (monitoring/jobs.py: check_holding_period_expiries)
- [x] Mako conflict detection (monitoring/jobs.py: check_new_mako_conflicts)
- [x] Runner script (scripts/run_monitoring.py: supports --run-now, --job <type>, or scheduler mode)

---

## Phase 5: Enhanced Slack Bot Features

### 5.1 New User Commands

| Command | Description | Example |
|---------|-------------|---------|
| `my positions` | Show current holdings from approved PAD requests | Lists all securities with quantities |
| `my requests` | Show PAD request history with status | Last 10 requests with status |
| `request status <id>` | Check specific request status | Full details of request #123 |
| `holding period <ticker>` | Check holding period for a security | "NVDA: 15 days remaining" |
| `can I trade <ticker>` | Quick pre-check before submitting | Checks conflicts, holding period |

### 5.2 Position Query

```
User: my positions

Bot: *Your Current Positions (from PAD history)*

| Security | Shares | Last Trade |
|----------|--------|------------|
| AAPL US | 100 | 2025-11-15 |
| NVDA US | 50 | 2025-10-20 |
| MSFT US | 200 | 2025-09-05 |

_Holding period status:_
- AAPL US: Can sell (held 33 days)
- NVDA US: Can sell (held 59 days)
- MSFT US: Can sell (held 104 days)

Type `holding period <ticker>` for details.
```

### 5.3 Request History

```
User: my requests

Bot: *Your Recent PAD Requests*

| ID | Date | Security | Action | Status |
|----|------|----------|--------|--------|
| #156 | Dec 17 | NVDA US | BUY 100 | Pending Compliance |
| #142 | Dec 10 | AAPL US | BUY 50 | Approved |
| #138 | Dec 05 | TSLA US | SELL 25 | Executed |
| #125 | Nov 28 | MSFT US | BUY 100 | Declined |

Type `request status <id>` for details.
```

### 5.4 Pre-Trade Check

```
User: can I trade NVDA

Bot: *Pre-Trade Check: NVDA US (NVIDIA Corp)*

*Compliance Checks:*
- Prohibited instrument: No
- Restricted list: No
- Mako conflict: Yes - Mako traded NVDA 15 days ago
- Your holding period: N/A (no existing position)

*Risk Assessment:*
- Likely classification: MEDIUM (Mako conflict)
- Will require: Compliance approval

*Recommendation:* You can submit a request, but it will need manual review due to Mako trading activity.

Would you like to proceed? Say "buy X shares of NVDA" to start.
```

### 5.5 Deliverables
- [x] `my positions` command (chatbot.py: get_my_positions, tools.py: get_all_employee_positions)
- [x] `my requests` command (chatbot.py: get_my_requests, tools.py: get_recent_pad_requests)
- [x] `request status <id>` command (chatbot.py: get_request_status - existing tool)
- [x] `holding period <ticker>` command (chatbot.py: check_holding_period, policy_engine.py: EnhancedHoldingPeriodChecker)
- [x] `can I trade <ticker>` pre-check command (chatbot.py: pre_trade_check, tools.py: pre_trade_check)
- [x] Help command with all available options (chatbot.py: get_help)

---

## Phase 6: Audit & Compliance Dashboard

### 6.1 Comprehensive Audit Trail

Every action logged with:
- Timestamp
- Actor (employee ID, system, AI)
- Action type
- Entity affected
- Before/after state
- AI prompt and response (if applicable)
- Source (Slack, API, scheduled job)

### 6.2 Compliance Dashboard (Web UI)

**Views needed:**
1. **Pending Approvals** - Requests awaiting manager/compliance/SMF16 action
2. **Breach Alerts** - Active breaches requiring attention
3. **Execution Tracking** - Approved requests pending execution
4. **Holding Period Calendar** - When employees can sell
5. **Mako Conflict Monitor** - Employee positions vs Mako trading
6. **Audit Log Search** - Full audit trail query interface
7. **Reports** - Monthly/quarterly PAD activity reports

### 6.3 Deliverables
- [ ] Audit log schema and logging service
- [ ] Dashboard API endpoints
- [ ] Dashboard UI (React/Next.js or similar)
- [ ] Standard compliance reports

---

## Implementation Phases

### Phase 1: Foundation (Week 1-2) ✅ COMPLETE
- Schema redesign and migration scripts
- Fix instrument identification
- Fix state management
- Migrate test data

### Phase 2: Policy Engine (Week 3-4) ✅ COMPLETE
- Prohibited instruments detection
- Exemptions engine
- Enhanced holding period logic
- Store all check results

### Phase 3: AI Enhancement (Week 5-6) ✅ COMPLETE
- Full risk scoring implementation
- Auto-approve flow
- SMF16 escalation
- AI decision logging

### Phase 4: Post-Trade Monitoring (Week 7-8) ✅ COMPLETE
- [x] Execution confirmation Slack flow
- [x] Contract note tracking (30-day deadline breach detection)
- [x] Deadline tracking scheduled job (2 business day deadline)
- [x] Breach detection scheduled jobs (4 job types)
- [x] Compliance alert notifications (Slack channel + employee DMs)
- [x] Runner script (scheduler mode + immediate mode)

### Phase 5: Slack Features (Week 9-10) ✅ COMPLETE
- Position query commands (my positions)
- Request history commands (my requests)
- Pre-trade check command (can I trade)
- Holding period check command
- Help command

### Phase 6: Dashboard (Week 11-12)
- API development
- UI implementation
- Reports
- Final testing

---

## Success Criteria (from Spec)

- [ ] 90% of low-risk requests correctly identified and auto-approved
- [ ] Zero breaches due to automation errors
- [ ] 100% of approvals logged with full AI and data trace
- [ ] 95% correct classification of prohibited cases
- [ ] 97% AI-detected holding period breaches
- [ ] Near-zero false approvals of prohibited trades

---

## Open Questions - RESOLVED

1. **Auto-approve threshold:** RESOLVED - Configurable in DB via `compliance_config` table. Default $10,000.
2. **SMF16 escalation:** TBD - Will define criteria once core workflow is working.
3. **Contract note format:** RESOLVED - Employee uploads PDF of broker confirmation after trade execution. System validates trade matches approval (within 5% tolerance).
4. **Mako position data:** RESOLVED - Assume always up-to-date (refreshed regularly by existing process).
5. **Dashboard access:** RESOLVED - Compliance only for now.

---

## Appendix: Migration Mapping

| Old Field | New Location | Notes |
|-----------|-------------|-------|
| ID | pad_request.id | Direct map |
| REQUESTED_DATE | pad_request.created_at | Rename |
| EMPLOYEE_ID | pad_request.employee_id | Direct map |
| BLOOMBERG | security.bloomberg | Normalize, strip " Equity" |
| INST_SYMBOL | security.ticker | Map to ticker |
| SECURITY_DESCRIPTION | security.description | Move to security table |
| BUYSELL | pad_request.direction | Map B->BUY, S->SELL |
| TRADE_SIZE | pad_request.quantity | Rename |
| VALUE | pad_request.estimated_value | Rename |
| STATUS | pad_request.status | Map to new status enum |
| STATUS_MANAGER | pad_approval (manager row) | Normalize |
| STATUS_COMPLIANCE | pad_approval (compliance row) | Normalize |
| AUTH_MANAGER_* | pad_approval (manager row) | Normalize |
| AUTH_COMPLIANCE_* | pad_approval (compliance row) | Normalize |
| HOLDING_PERIOD_YN | pad_approval.holding_period_check | Move to approval |
| RESTRICTED_YN | pad_approval.restricted_check | Move to approval |
| CONFLICTS_YN | pad_approval.conflict_check | Move to approval |

---

*End of Document*
