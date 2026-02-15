# Implementation Plan: PAD Spec Compliance Gaps

## Overview

This plan addresses gaps identified between the PAD specification and implementation. Scope is focused on practical improvements, not theoretical compliance theatre.

**Out of Scope:**
- Uptime monitoring (handled by cloud platform/k8s)
- Accuracy "tests" (we'll build dashboards from recorded data instead)

---

## Gap 1: Enforce 7-Year Audit Retention

### Problem

The monitoring job deletes audit logs after 90 days:
```python
# jobs.py:969
audit_cutoff = now - timedelta(days=90)
```

This violates the regulatory requirement for 7-year retention.

### Solution

Remove audit log cleanup entirely, or hardcode 7-year minimum that cannot be configured lower.

### Implementation

**File**: `src/pa_dealing/agents/monitoring/jobs.py`

**Change 1**: Find the audit cleanup section (around line 965-975) and either:

Option A - Remove audit cleanup entirely:
```python
# REMOVED: Audit logs must be retained for 7 years per regulatory requirements
# audit_cutoff = now - timedelta(days=90)
# await session.execute(
#     delete(AuditLog).where(AuditLog.created_at < audit_cutoff)
# )
```

Option B - Enforce 7-year minimum:
```python
# Regulatory requirement: 7-year retention minimum
AUDIT_RETENTION_YEARS = 7
AUDIT_RETENTION_DAYS = AUDIT_RETENTION_YEARS * 365  # 2555 days

audit_cutoff = now - timedelta(days=AUDIT_RETENTION_DAYS)
deleted = await session.execute(
    delete(AuditLog).where(AuditLog.created_at < audit_cutoff)
)
logger.info(f"Cleaned up {deleted.rowcount} audit logs older than {AUDIT_RETENTION_YEARS} years")
```

**Change 2**: Add comment explaining the regulatory requirement:
```python
# IMPORTANT: 7-year audit retention is a regulatory requirement.
# Do not reduce this value. See PAD Spec section 8: Security, Privacy and Compliance.
```

**File**: `src/pa_dealing/config/settings.py`

Do NOT add a configurable setting for this. The retention period should be hardcoded to prevent accidental misconfiguration.

### Success Criteria
- [x] Audit logs older than 90 days are no longer deleted
- [x] Code comment explains regulatory requirement
- [x] No configuration option exists to reduce retention below 7 years

---

## Gap 2: Accuracy Metrics Dashboard Data

### Problem

The spec requires proving accuracy:
- 95% correct classification of prohibited cases
- 97% AI detected holding period breaches
- 90% match to historical compliance decisions

We don't have "tests" for this - we need to **record the data** so compliance can see a dashboard showing these metrics.

### Solution

Record decision outcomes and actual results so we can calculate accuracy metrics retrospectively.

### What Data We Need to Record

| Metric | What to Record | When |
|--------|---------------|------|
| Prohibited classification accuracy | AI said prohibited? Compliance agreed? | On every prohibited flag |
| Holding period detection accuracy | AI detected violation? Was it real? | On every holding period check |
| Decision match rate | AI recommendation vs final human decision | On every approval/decline |

### Implementation

**File**: Create `src/pa_dealing/db/models.py` - Add new table

```python
class ComplianceDecisionOutcome(Base):
    """
    Records AI decisions and actual outcomes for accuracy tracking.

    This data powers the compliance accuracy dashboard showing:
    - Prohibited classification accuracy
    - Holding period detection accuracy
    - AI recommendation vs human decision match rate
    """
    __tablename__ = "compliance_decision_outcome"

    id: Mapped[int] = mapped_column(primary_key=True)
    request_id: Mapped[int] = mapped_column(ForeignKey("pad_request.id"), index=True)

    # What the AI decided
    ai_risk_level: Mapped[str] = mapped_column(String(20))  # LOW/MEDIUM/HIGH
    ai_recommendation: Mapped[str] = mapped_column(String(20))  # APPROVE/REVIEW/REJECT
    ai_prohibited_flag: Mapped[bool] = mapped_column(default=False)
    ai_holding_period_violation: Mapped[bool] = mapped_column(default=False)
    ai_restricted_flag: Mapped[bool] = mapped_column(default=False)
    ai_conflict_level: Mapped[str | None] = mapped_column(String(20))  # none/low/medium/high
    ai_score: Mapped[int] = mapped_column(default=0)

    # What actually happened (filled in after human review)
    final_decision: Mapped[str | None] = mapped_column(String(20))  # approved/declined
    human_override: Mapped[bool] = mapped_column(default=False)  # Did human disagree with AI?
    override_reason: Mapped[str | None] = mapped_column(Text)

    # For prohibited accuracy
    confirmed_prohibited: Mapped[bool | None] = mapped_column()  # Compliance confirmed?

    # For holding period accuracy
    confirmed_holding_violation: Mapped[bool | None] = mapped_column()  # Was it real?

    # Timestamps
    ai_decision_at: Mapped[datetime] = mapped_column(default=datetime.utcnow)
    final_decision_at: Mapped[datetime | None] = mapped_column()

    # Relationships
    request: Mapped["PADRequest"] = relationship(back_populates="decision_outcome")
```

**File**: `src/pa_dealing/agents/orchestrator/agent.py`

After risk classification, record the AI decision:

```python
# After line ~400 where risk classification happens
async def _record_decision_outcome(
    self,
    session: AsyncSession,
    request_id: int,
    classification: RiskClassification,
    compliance_checks: list[ComplianceCheckResult]
) -> None:
    """Record AI decision for accuracy tracking dashboard."""
    from ..db.models import ComplianceDecisionOutcome

    # Extract key flags from checks
    prohibited = any(c.check_type == "prohibited" and not c.passed for c in compliance_checks)
    holding_violation = any(c.check_type == "holding_period" and not c.passed for c in compliance_checks)
    restricted = any(c.check_type == "restricted" and not c.passed for c in compliance_checks)
    conflict = next((c.details.get("conflict_level") for c in compliance_checks if c.check_type == "mako_conflict"), None)

    outcome = ComplianceDecisionOutcome(
        request_id=request_id,
        ai_risk_level=classification.level.value,
        ai_recommendation=self._map_to_recommendation(classification),
        ai_prohibited_flag=prohibited,
        ai_holding_period_violation=holding_violation,
        ai_restricted_flag=restricted,
        ai_conflict_level=conflict,
        ai_score=classification.score,
    )
    session.add(outcome)

def _map_to_recommendation(self, classification: RiskClassification) -> str:
    """Map risk classification to APPROVE/REVIEW/REJECT."""
    if classification.auto_approve_eligible:
        return "APPROVE"
    elif classification.requires_smf16_escalation:
        return "REJECT"  # Requires SMF16 override
    else:
        return "REVIEW"
```

**File**: `src/pa_dealing/agents/database/tools.py`

Add function to update outcome after human decision:

```python
async def record_decision_outcome(
    session: AsyncSession,
    request_id: int,
    final_decision: str,
    human_override: bool = False,
    override_reason: str | None = None,
    confirmed_prohibited: bool | None = None,
    confirmed_holding_violation: bool | None = None,
) -> None:
    """
    Update the decision outcome record after human review.

    Called when:
    - Manager/Compliance approves or declines
    - Compliance confirms/disputes AI flags
    """
    from datetime import datetime
    from ..db.models import ComplianceDecisionOutcome

    result = await session.execute(
        select(ComplianceDecisionOutcome).where(
            ComplianceDecisionOutcome.request_id == request_id
        )
    )
    outcome = result.scalar_one_or_none()

    if outcome:
        outcome.final_decision = final_decision
        outcome.final_decision_at = datetime.utcnow()
        outcome.human_override = human_override
        outcome.override_reason = override_reason
        if confirmed_prohibited is not None:
            outcome.confirmed_prohibited = confirmed_prohibited
        if confirmed_holding_violation is not None:
            outcome.confirmed_holding_violation = confirmed_holding_violation
```

**File**: `src/pa_dealing/api/routes/dashboard.py`

Add accuracy metrics endpoint for the dashboard:

```python
@router.get("/accuracy-metrics")
async def get_accuracy_metrics(
    session: SessionDep,
    current_user: CurrentUserDep,
    days: int = 90,
) -> dict:
    """
    Get accuracy metrics for compliance dashboard.

    Returns:
    - Prohibited classification accuracy (target: 95%)
    - Holding period detection accuracy (target: 97%)
    - AI recommendation match rate (target: 90%)
    """
    from datetime import datetime, timedelta
    from sqlalchemy import func, and_

    cutoff = datetime.utcnow() - timedelta(days=days)

    # Prohibited accuracy: AI flagged prohibited AND compliance confirmed
    prohibited_stats = await session.execute(
        select(
            func.count().filter(ComplianceDecisionOutcome.ai_prohibited_flag == True).label("ai_flagged"),
            func.count().filter(
                and_(
                    ComplianceDecisionOutcome.ai_prohibited_flag == True,
                    ComplianceDecisionOutcome.confirmed_prohibited == True
                )
            ).label("confirmed_correct"),
            func.count().filter(
                and_(
                    ComplianceDecisionOutcome.ai_prohibited_flag == False,
                    ComplianceDecisionOutcome.confirmed_prohibited == True
                )
            ).label("false_negatives"),
        ).where(ComplianceDecisionOutcome.ai_decision_at >= cutoff)
    )
    prohibited = prohibited_stats.one()

    # Holding period accuracy
    holding_stats = await session.execute(
        select(
            func.count().filter(ComplianceDecisionOutcome.ai_holding_period_violation == True).label("ai_flagged"),
            func.count().filter(
                and_(
                    ComplianceDecisionOutcome.ai_holding_period_violation == True,
                    ComplianceDecisionOutcome.confirmed_holding_violation == True
                )
            ).label("confirmed_correct"),
        ).where(ComplianceDecisionOutcome.ai_decision_at >= cutoff)
    )
    holding = holding_stats.one()

    # Decision match rate
    match_stats = await session.execute(
        select(
            func.count().label("total_decisions"),
            func.count().filter(ComplianceDecisionOutcome.human_override == False).label("matches"),
        ).where(
            and_(
                ComplianceDecisionOutcome.ai_decision_at >= cutoff,
                ComplianceDecisionOutcome.final_decision.isnot(None)
            )
        )
    )
    match = match_stats.one()

    return {
        "period_days": days,
        "prohibited_classification": {
            "ai_flagged_count": prohibited.ai_flagged,
            "confirmed_correct": prohibited.confirmed_correct,
            "false_negatives": prohibited.false_negatives,
            "accuracy_pct": round(prohibited.confirmed_correct / max(prohibited.ai_flagged, 1) * 100, 1),
            "target_pct": 95,
        },
        "holding_period_detection": {
            "ai_flagged_count": holding.ai_flagged,
            "confirmed_correct": holding.confirmed_correct,
            "accuracy_pct": round(holding.confirmed_correct / max(holding.ai_flagged, 1) * 100, 1),
            "target_pct": 97,
        },
        "decision_match_rate": {
            "total_decisions": match.total_decisions,
            "ai_human_matches": match.matches,
            "match_rate_pct": round(match.matches / max(match.total_decisions, 1) * 100, 1),
            "target_pct": 90,
        },
    }
```

### Dashboard UI Component

**File**: `dashboard/src/pages/AccuracyMetrics.tsx` (new file)

Create a dashboard page showing these metrics with:
- Current accuracy percentages vs targets
- Trend over time (last 7/30/90 days)
- Breakdown by decision type
- List of overridden decisions for review

### Success Criteria
- [x] `ComplianceDecisionOutcome` table created
- [x] AI decisions recorded on every request
- [x] Human decisions update the outcome record
- [x] `/api/accuracy-metrics` endpoint returns metrics
- [x] Dashboard displays accuracy vs targets

---

## Gap 3: Ensure Response Contains All Required Data

### Problem

The spec implies certain data should be in responses. Current response is more detailed but structured differently. We should ensure no implied functionality is missing.

### Analysis of Spec Output

```json
{
  "risk_classification": "LOW/MEDIUM/HIGH",
  "recommendation": "APPROVE/REVIEW/REJECT",
  "policy_flags": [...],
  "explanation": "...",
  "suggested_approver": "COMPLIANCE/SMF16"
}
```

**What we have vs what's implied:**

| Spec Field | Current Implementation | Gap? |
|------------|----------------------|------|
| `risk_classification` | ✅ `risk_classification.level` | No |
| `recommendation` | ❌ Not explicit | Yes - add mapping |
| `policy_flags` | ✅ In `factors` array | No - just named differently |
| `explanation` | ✅ `ComplianceRationale.summary` | No |
| `suggested_approver` | ❌ Boolean flags only | Yes - add explicit field |

### Implementation

**File**: `src/pa_dealing/agents/orchestrator/schemas.py`

Add explicit recommendation and suggested_approver to response:

```python
class RecommendationType(str, Enum):
    """Explicit recommendation matching spec."""
    APPROVE = "APPROVE"  # Auto-approve eligible
    REVIEW = "REVIEW"    # Requires compliance review
    REJECT = "REJECT"    # Requires SMF16 override or auto-declined

class SuggestedApprover(str, Enum):
    """Who should approve this request."""
    AUTO = "AUTO"              # Can be auto-approved
    MANAGER = "MANAGER"        # Line manager
    COMPLIANCE = "COMPLIANCE"  # Compliance team
    SMF16 = "SMF16"           # SMF16 holder required

class RiskAssessmentResponse(BaseModel):
    """Response structure for risk assessment."""
    # Core classification
    risk_level: RiskLevel
    risk_score: int

    # Explicit recommendation (maps to spec)
    recommendation: RecommendationType
    suggested_approver: SuggestedApprover

    # Policy flags (what spec calls policy_flags)
    policy_flags: list[str]

    # Explanation
    explanation: str

    # Detailed breakdown (our extended data)
    factors: list[str]
    compliance_checks: list[dict]
    can_proceed: bool
    warnings: list[str]

    # Metadata
    auto_approve_eligible: bool
    requires_smf16_escalation: bool
```

**File**: `src/pa_dealing/agents/orchestrator/agent.py`

Update `classify_risk` to return the new structure:

```python
def _determine_recommendation(self, classification: RiskClassification) -> RecommendationType:
    """Map classification to explicit recommendation."""
    if classification.auto_approve_eligible:
        return RecommendationType.APPROVE
    elif classification.requires_smf16_escalation or classification.level == RiskLevel.HIGH:
        return RecommendationType.REJECT
    else:
        return RecommendationType.REVIEW

def _determine_approver(self, classification: RiskClassification) -> SuggestedApprover:
    """Determine who should approve."""
    if classification.auto_approve_eligible:
        return SuggestedApprover.AUTO
    elif classification.requires_smf16_escalation:
        return SuggestedApprover.SMF16
    elif classification.level == RiskLevel.HIGH:
        return SuggestedApprover.SMF16
    elif classification.level == RiskLevel.MEDIUM:
        return SuggestedApprover.COMPLIANCE
    else:
        return SuggestedApprover.MANAGER
```

### Success Criteria
- [x] Response includes explicit `recommendation` field
- [x] Response includes explicit `suggested_approver` field
- [x] `policy_flags` is a top-level field (not nested)
- [x] `explanation` is a top-level string field

---

## Gap 4: Add Response Time Logging

### Problem

Spec requires < 2 second response time. We don't measure this currently.

### Solution

Add timing instrumentation to log response times. No alerting - just capture data for analysis.

### Implementation

**File**: `src/pa_dealing/api/middleware.py` (create if doesn't exist)

```python
"""API middleware for request timing and logging."""
import time
import logging
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request

logger = logging.getLogger(__name__)

class ResponseTimeMiddleware(BaseHTTPMiddleware):
    """Log response times for all API requests."""

    async def dispatch(self, request: Request, call_next):
        start_time = time.perf_counter()

        response = await call_next(request)

        duration_ms = (time.perf_counter() - start_time) * 1000

        # Log timing
        logger.info(
            "request_timing",
            extra={
                "path": request.url.path,
                "method": request.method,
                "duration_ms": round(duration_ms, 2),
                "status_code": response.status_code,
            }
        )

        # Add header for debugging
        response.headers["X-Response-Time-Ms"] = str(round(duration_ms, 2))

        return response
```

**File**: `src/pa_dealing/api/main.py`

Add the middleware:

```python
from .middleware import ResponseTimeMiddleware

# After creating app
app.add_middleware(ResponseTimeMiddleware)
```

**File**: `src/pa_dealing/agents/orchestrator/agent.py`

Add timing to the critical `process_pad_request` method:

```python
import time

async def process_pad_request(self, ...) -> dict:
    """Process a PAD request with timing instrumentation."""
    timings = {}
    total_start = time.perf_counter()

    # Employee lookup
    t0 = time.perf_counter()
    employee = await self._get_employee(...)
    timings["employee_lookup_ms"] = (time.perf_counter() - t0) * 1000

    # Policy checks
    t0 = time.perf_counter()
    policy_result = await self._run_policy_checks(...)
    timings["policy_checks_ms"] = (time.perf_counter() - t0) * 1000

    # Risk classification
    t0 = time.perf_counter()
    classification = self._classify_risk(...)
    timings["risk_classification_ms"] = (time.perf_counter() - t0) * 1000

    # Database submission
    t0 = time.perf_counter()
    request_id = await self._submit_request(...)
    timings["db_submission_ms"] = (time.perf_counter() - t0) * 1000

    timings["total_ms"] = (time.perf_counter() - total_start) * 1000

    logger.info(
        "pad_request_processed",
        extra={
            "request_id": request_id,
            "risk_level": classification.level.value,
            "timings": timings,
        }
    )

    return {
        ...,
        "_timings": timings,  # Include in response for debugging
    }
```

### Success Criteria
- [x] All API requests log response time
- [x] `X-Response-Time-Ms` header added to responses
- [x] `process_pad_request` logs breakdown of time spent
- [x] Timings visible in application logs

---

## Gap 6: Insider Information Enforcement

### Problem

The spec requires: "Requestor confirms no inside information"

Currently this is only mentioned in chatbot instructions. There's no:
- Database field to record the declaration
- Blocking logic if insider info is indicated
- Audit trail of the declaration

### Solution

Add explicit insider information handling:
1. Add field to capture declaration
2. Block request if insider info indicated
3. Audit log the declaration

### Implementation

**File**: `src/pa_dealing/db/models.py`

Add field to PADRequest:

```python
class PADRequest(Base):
    # ... existing fields ...

    # Insider information declaration
    # True = User confirms NO insider information (safe to proceed)
    # False = User indicated they MAY have insider information (block)
    # None = Not yet declared (legacy requests)
    insider_info_declaration: Mapped[bool | None] = mapped_column(
        Boolean,
        nullable=True,
        comment="True if user confirms NO insider info, False if they may have insider info"
    )
    insider_info_declared_at: Mapped[datetime | None] = mapped_column()
```

**File**: `src/pa_dealing/agents/slack/handlers.py`

Update modal to include insider info checkbox:

```python
# In _handle_open_modal, add to the blocks list:
{
    "type": "input",
    "block_id": "insider_block",
    "element": {
        "type": "checkboxes",
        "action_id": "insider_input",
        "options": [
            {
                "text": {
                    "type": "plain_text",
                    "text": "I confirm I do not possess any inside information relating to this security",
                },
                "value": "no_insider_info",
            },
        ],
    },
    "label": {"type": "plain_text", "text": "Insider Information Declaration *"},
},
```

Update `_handle_view_submission` to extract and validate:

```python
# Extract insider declaration
insider_options = values.get("insider_block", {}).get("insider_input", {}).get("selected_options", [])
has_confirmed_no_insider = any(o["value"] == "no_insider_info" for o in insider_options)

if not has_confirmed_no_insider:
    # Block the request - cannot proceed without declaration
    await self.web_client.chat_postMessage(
        channel=target_channel,
        text=":x: *Request Blocked*\n\nYou must confirm that you do not possess inside information relating to this security before submitting a PAD request.",
        thread_ts=slack_thread_ts,
    )
    return

# Pass to orchestrator
result = await orchestrator.process_pad_request(
    ...,
    insider_info_confirmed=has_confirmed_no_insider,
)
```

**File**: `src/pa_dealing/agents/orchestrator/agent.py`

Update `process_pad_request` signature and handling:

```python
async def process_pad_request(
    self,
    ...,
    insider_info_confirmed: bool = False,  # New parameter
) -> dict:
    # Early block if insider info not confirmed
    if not insider_info_confirmed:
        logger.warning(f"PAD request blocked: insider info not confirmed by {employee_email}")

        await audit.log(
            action_type=ActionType.PAD_REQUEST_SUBMITTED,
            action_status=ActionStatus.BLOCKED,
            actor_email=employee_email,
            details={
                "reason": "insider_info_not_confirmed",
                "security": security_description,
            }
        )

        return {
            "status": "blocked",
            "reason": "Insider information declaration required",
            "can_proceed": False,
        }

    # Continue with normal processing...
    # Record the declaration on the request
```

**File**: `src/pa_dealing/agents/slack/chatbot.py`

Update chatbot to ask about insider info during conversational flow:

```python
# Add to the chatbot instructions/prompts:
"""
IMPORTANT: Before submitting any PAD request, you MUST confirm with the user:
"Do you confirm that you do not possess any inside information relating to this security?"

If the user indicates they may have inside information, or refuses to confirm:
- DO NOT submit the request
- Inform them they cannot trade while in possession of inside information
- Suggest they speak with Compliance if they have questions

Only proceed with submission after receiving explicit confirmation.
"""
```

**File**: `src/pa_dealing/api/routes/requests.py`

Update API endpoint for direct submissions:

```python
class PADRequestCreate(BaseModel):
    # ... existing fields ...
    insider_info_confirmed: bool = Field(
        ...,
        description="User confirms they do not possess inside information"
    )

@router.post("/requests")
async def create_request(
    request: PADRequestCreate,
    ...
):
    if not request.insider_info_confirmed:
        raise HTTPException(
            status_code=400,
            detail="Insider information declaration is required"
        )
    # Continue...
```

### Success Criteria
- [x] `insider_info_declaration` field added to PADRequest model
- [x] Slack modal requires insider info checkbox
- [x] Chatbot asks for confirmation before submission
- [x] API endpoint requires `insider_info_confirmed`
- [x] Requests blocked if declaration not provided
- [x] Audit log records the declaration

---

## Gap 7: MAR/MiFID Rule Detection

### Problem

The spec requires: "AI must not approve if breach of MAR/MiFID inferred"

Currently only holding period is checked. Missing:
- MAR window period detection (trading around price-sensitive announcements)
- Excessive trading patterns that might indicate market manipulation
- Cross-trade detection (employee trading opposite to Mako)

### Solution

Add additional compliance checks for MAR-related patterns.

### Implementation

**File**: `src/pa_dealing/agents/orchestrator/policy_engine.py`

Add new detector class:

```python
class MARComplianceDetector:
    """
    Detects potential Market Abuse Regulation (MAR) concerns.

    Checks for:
    1. Trading in securities where Mako has recent activity (front-running risk)
    2. Opposite-direction trades to Mako (potential cross-trading)
    3. Trading patterns suggesting market timing
    4. Repeated short-term trades in same security
    """

    def __init__(self, config: ComplianceThresholds):
        self.config = config
        # MAR-specific thresholds
        self.mako_trade_window_days = 7  # Flag if Mako traded within N days
        self.opposite_trade_window_days = 3  # Flag opposite trades within N days
        self.pattern_lookback_days = 30
        self.pattern_trade_threshold = 5  # N trades in same security = pattern

    async def check(
        self,
        session: AsyncSession,
        employee_id: int,
        ticker: str,
        direction: str,  # "B" or "S"
        **kwargs
    ) -> MARCheckResult:
        """
        Run MAR compliance checks.

        Returns MARCheckResult with:
        - is_flagged: bool
        - flags: list of specific MAR concerns
        - severity: LOW/MEDIUM/HIGH
        - recommendation: str
        """
        flags = []
        severity = "LOW"

        # Check 1: Mako recent trading activity
        mako_activity = await self._check_mako_recent_activity(session, ticker)
        if mako_activity["traded_recently"]:
            flags.append({
                "type": "MAKO_RECENT_TRADE",
                "message": f"Mako traded {ticker} {mako_activity['days_ago']} days ago",
                "severity": "MEDIUM" if mako_activity["days_ago"] <= 3 else "LOW",
            })

        # Check 2: Opposite direction to Mako
        opposite = await self._check_opposite_direction(session, ticker, direction)
        if opposite["is_opposite"]:
            flags.append({
                "type": "OPPOSITE_DIRECTION",
                "message": f"Employee {direction} while Mako recently {'sold' if direction == 'B' else 'bought'}",
                "severity": "HIGH",
            })
            severity = "HIGH"

        # Check 3: Repeated trades in same security (pattern)
        pattern = await self._check_trading_pattern(session, employee_id, ticker)
        if pattern["is_pattern"]:
            flags.append({
                "type": "TRADING_PATTERN",
                "message": f"Employee has made {pattern['trade_count']} trades in {ticker} in last {self.pattern_lookback_days} days",
                "severity": "MEDIUM",
            })
            if severity == "LOW":
                severity = "MEDIUM"

        # Check 4: Short-term round trips (buy then sell or vice versa within days)
        round_trip = await self._check_round_trip(session, employee_id, ticker, direction)
        if round_trip["is_round_trip"]:
            flags.append({
                "type": "SHORT_TERM_ROUND_TRIP",
                "message": f"Potential round-trip: {round_trip['description']}",
                "severity": "HIGH",
            })
            severity = "HIGH"

        return MARCheckResult(
            is_flagged=len(flags) > 0,
            flags=flags,
            severity=severity,
            recommendation=self._generate_recommendation(flags, severity),
        )

    async def _check_mako_recent_activity(self, session, ticker: str) -> dict:
        """Check if Mako has traded this security recently."""
        from datetime import datetime, timedelta

        cutoff = datetime.utcnow() - timedelta(days=self.mako_trade_window_days)

        # Query Mako trading activity (assumes we have this data)
        result = await session.execute(
            select(OraclePosition)
            .where(OraclePosition.inst_symbol == ticker)
            .where(OraclePosition.last_trade_date >= cutoff)
        )
        position = result.scalar_one_or_none()

        if position and position.last_trade_date:
            days_ago = (datetime.utcnow() - position.last_trade_date).days
            return {"traded_recently": True, "days_ago": days_ago}

        return {"traded_recently": False, "days_ago": None}

    async def _check_opposite_direction(self, session, ticker: str, employee_direction: str) -> dict:
        """Check if employee is trading opposite to recent Mako activity."""
        from datetime import datetime, timedelta

        cutoff = datetime.utcnow() - timedelta(days=self.opposite_trade_window_days)

        # Get Mako's recent direction for this security
        # This requires tracking Mako trade direction - may need schema update
        result = await session.execute(
            select(OraclePosition)
            .where(OraclePosition.inst_symbol == ticker)
        )
        position = result.scalar_one_or_none()

        if position:
            # Infer Mako direction from position change
            # Positive position = Mako is long (bought)
            # If employee is selling while Mako bought = opposite
            mako_is_long = position.position_size > 0
            employee_is_buying = employee_direction == "B"

            # Opposite if: Mako long and employee selling, or Mako short and employee buying
            is_opposite = (mako_is_long and not employee_is_buying) or (not mako_is_long and employee_is_buying)

            if is_opposite:
                return {
                    "is_opposite": True,
                    "mako_direction": "BUY" if mako_is_long else "SELL",
                    "employee_direction": "BUY" if employee_is_buying else "SELL",
                }

        return {"is_opposite": False}

    async def _check_trading_pattern(self, session, employee_id: int, ticker: str) -> dict:
        """Check for repeated trades in same security."""
        from datetime import datetime, timedelta

        cutoff = datetime.utcnow() - timedelta(days=self.pattern_lookback_days)

        result = await session.execute(
            select(func.count())
            .select_from(PADRequest)
            .where(PADRequest.employee_id == employee_id)
            .where(PADRequest.ticker == ticker)
            .where(PADRequest.created_at >= cutoff)
            .where(PADRequest.status.in_(["approved", "executed"]))
        )
        count = result.scalar()

        return {
            "is_pattern": count >= self.pattern_trade_threshold,
            "trade_count": count,
        }

    async def _check_round_trip(self, session, employee_id: int, ticker: str, direction: str) -> dict:
        """Check for short-term round-trip trades."""
        from datetime import datetime, timedelta

        # Look for opposite trade in last 14 days
        cutoff = datetime.utcnow() - timedelta(days=14)
        opposite_direction = "S" if direction == "B" else "B"

        result = await session.execute(
            select(PADRequest)
            .where(PADRequest.employee_id == employee_id)
            .where(PADRequest.ticker == ticker)
            .where(PADRequest.direction == opposite_direction)
            .where(PADRequest.created_at >= cutoff)
            .where(PADRequest.status.in_(["approved", "executed"]))
            .order_by(PADRequest.created_at.desc())
            .limit(1)
        )
        recent_opposite = result.scalar_one_or_none()

        if recent_opposite:
            days_ago = (datetime.utcnow() - recent_opposite.created_at).days
            return {
                "is_round_trip": True,
                "description": f"{'Bought' if opposite_direction == 'B' else 'Sold'} {ticker} {days_ago} days ago, now {'selling' if direction == 'S' else 'buying'}",
                "days_since_opposite": days_ago,
            }

        return {"is_round_trip": False}

    def _generate_recommendation(self, flags: list, severity: str) -> str:
        """Generate recommendation based on MAR flags."""
        if severity == "HIGH":
            return "ESCALATE_SMF16: Multiple MAR concerns detected. Requires SMF16 review."
        elif severity == "MEDIUM":
            return "COMPLIANCE_REVIEW: MAR-related patterns detected. Manual compliance review recommended."
        elif flags:
            return "PROCEED_WITH_NOTE: Minor MAR flags noted. Proceed with standard approval."
        else:
            return "PROCEED: No MAR concerns detected."
```

**File**: `src/pa_dealing/agents/orchestrator/schemas.py`

Add schema for MAR check result:

```python
class MARCheckResult(BaseModel):
    """Result of MAR compliance check."""
    is_flagged: bool
    flags: list[dict]
    severity: str  # LOW/MEDIUM/HIGH
    recommendation: str
```

**File**: `src/pa_dealing/agents/orchestrator/agent.py`

Integrate MAR checks into the workflow:

```python
async def process_pad_request(self, ...):
    # ... existing checks ...

    # MAR/MiFID compliance check
    mar_detector = MARComplianceDetector(self.config.thresholds)
    mar_result = await mar_detector.check(
        session=session,
        employee_id=employee.id,
        ticker=ticker,
        direction=buysell,
    )

    # Add MAR flags to risk factors
    if mar_result.is_flagged:
        for flag in mar_result.flags:
            risk_factors.append(f"MAR: {flag['message']}")

        # Escalate to SMF16 if HIGH severity
        if mar_result.severity == "HIGH":
            requires_smf16_escalation = True

    # Include in response
    analysis["mar_compliance"] = {
        "flagged": mar_result.is_flagged,
        "flags": mar_result.flags,
        "severity": mar_result.severity,
        "recommendation": mar_result.recommendation,
    }
```

### Success Criteria
- [x] `MARComplianceDetector` class created
- [x] Checks for Mako recent trading activity
- [x] Checks for opposite-direction trades
- [x] Checks for repeated trading patterns
- [x] Checks for short-term round trips
- [x] HIGH severity MAR flags trigger SMF16 escalation
- [x] MAR results included in response

---

## Alembic Migration

Create migration for new fields:

**File**: `alembic/versions/YYYYMMDD_spec_compliance_gaps.py`

```python
"""Add spec compliance gap fields.

Revision ID: spec_compliance_gaps
"""
from alembic import op
import sqlalchemy as sa

def upgrade():
    # Add insider information fields to pad_request
    op.add_column('pad_request', sa.Column('insider_info_declaration', sa.Boolean(), nullable=True))
    op.add_column('pad_request', sa.Column('insider_info_declared_at', sa.DateTime(), nullable=True))

    # Create compliance_decision_outcome table
    op.create_table(
        'compliance_decision_outcome',
        sa.Column('id', sa.Integer(), primary_key=True),
        sa.Column('request_id', sa.Integer(), sa.ForeignKey('pad_request.id'), index=True),
        sa.Column('ai_risk_level', sa.String(20)),
        sa.Column('ai_recommendation', sa.String(20)),
        sa.Column('ai_prohibited_flag', sa.Boolean(), default=False),
        sa.Column('ai_holding_period_violation', sa.Boolean(), default=False),
        sa.Column('ai_restricted_flag', sa.Boolean(), default=False),
        sa.Column('ai_conflict_level', sa.String(20), nullable=True),
        sa.Column('ai_score', sa.Integer(), default=0),
        sa.Column('final_decision', sa.String(20), nullable=True),
        sa.Column('human_override', sa.Boolean(), default=False),
        sa.Column('override_reason', sa.Text(), nullable=True),
        sa.Column('confirmed_prohibited', sa.Boolean(), nullable=True),
        sa.Column('confirmed_holding_violation', sa.Boolean(), nullable=True),
        sa.Column('ai_decision_at', sa.DateTime()),
        sa.Column('final_decision_at', sa.DateTime(), nullable=True),
    )

def downgrade():
    op.drop_table('compliance_decision_outcome')
    op.drop_column('pad_request', 'insider_info_declared_at')
    op.drop_column('pad_request', 'insider_info_declaration')
```

---

## Summary

| Gap | Solution | Priority | Effort |
|-----|----------|----------|--------|
| 7-year retention | Hardcode, remove 90-day cleanup | P0 | Low |
| Accuracy metrics | New table + dashboard endpoint | P1 | Medium |
| Response format | Add `recommendation` + `suggested_approver` | P2 | Low |
| Response timing | Middleware + method instrumentation | P2 | Low |
| Insider info | New field + modal checkbox + blocking | P1 | Medium |
| MAR/MiFID | New detector class + SMF16 escalation | P1 | High |

**Implementation Order:**
1. Gap 1 (retention) - Quick win, regulatory critical
2. Gap 6 (insider info) - Important compliance requirement
3. Gap 7 (MAR/MiFID) - Significant but needed for full compliance
4. Gap 2 (accuracy data) - Enables dashboard metrics
5. Gap 3 (response format) - Clean up
6. Gap 4 (timing) - Nice to have for debugging
