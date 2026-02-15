# Spec: Notification Reliability & Silent Failure Prevention

## Problem Statement

After a manager approves a PA dealing request, the compliance team should receive a Slack notification to review and approve. However, this notification is sometimes **silently lost** - the database shows the request as "pending_compliance" but compliance never receives the Slack message.

### Observed Behavior
1. User submits PAD request for BUND (Euro Bond), $200k, leveraged
2. Manager approves the request
3. Dashboard shows request as "Awaiting Compliance"
4. **Compliance Slack channel receives NO notification**
5. Request sits in limbo until manually discovered

### Root Cause Analysis
The notification code at `handlers.py:2385` discards the result of `send_message()`:
```python
await self._slack_client.send_message(request)  # Result not checked!
```

The `SlackMessageResult` has `success` and `error` fields, but they are ignored. If Slack fails (rate limit, network error, channel misconfiguration), the failure is silent.

### Additional Bugs (Phase 7 Deferred) - NOW FIXED
1. ~~**test_auto_approve_flow KeyError**: Orchestrator returns `{"success": False, "error": ...}` without "status" key~~ ✅ FIXED
2. ~~**test_insider_info oracle_fx**: Currency service queries `oracle_fx` table which isn't defined in SQLAlchemy models~~ ✅ FIXED

---

## CRITICAL BUG: Auto-Approve Bypassing Value Threshold (Added 2026-01-28)

### Observed Behavior (Production Incident)
A **€262,900** trade was auto-approved when it should have required manager/compliance approval:

```
User: buying 2,000 units... EUR 131.45 per unit, total EUR 262,900
Bot: Est. Value: USD 262,900.00
Bot: Request Approved - LOW risk, auto-approved by AI
```

The `auto_approve_max_value` default is **10,000** - this trade is **26x over the limit**.

### Two Sub-Issues

**Issue A: Auto-Approve Threshold Not Enforced**
- Trade value: 262,900
- Threshold: 10,000
- Expected: Require manager approval
- Actual: Auto-approved

**Issue B: Currency Ignored**
- User said: "EUR 262,900"
- System stored: "USD 262,900.00"
- Currency was not parsed/preserved from user input

### Investigation Required
1. **Why was auto-approve triggered?**
   - Is `factors.trade_value` being set correctly from chatbot input?
   - Is `auto_approve_max_value` config being loaded from DB?
   - Is the comparison `trade_value <= auto_approve_max_value` working correctly?
   - Check: `risk_classifier.py:520-523`

2. **Why was currency ignored?**
   - Is chatbot extracting currency from user message?
   - Is there a default currency fallback overwriting user input?
   - Check: `chatbot.py` value/currency parsing logic

### Reproduction Steps
1. Start PAD chatbot conversation
2. Say: "buying 2,000 units at EUR 131.45, total EUR 262,900"
3. Complete the flow (not derivative, not leveraged, confirm insider info)
4. Observe: Should require approval but gets auto-approved

### Severity: CRITICAL
This allows high-value trades to bypass approval workflow entirely

### Research Requirements (Before Implementation)

**1. Analyze Current Implementation**
- Review how the chatbot agent currently extracts currency and value from user messages
- Trace the data flow from user input → chatbot parsing → orchestrator → risk scoring
- Identify where currency information is lost or defaulted

**2. Research ADK Best Practices (Use Context7)**
- Query Context7 for Google ADK documentation on:
  - Structured data extraction from natural language
  - Currency/numeric parsing patterns
  - Agent tool design for financial data
- Research ADK's recommended approaches for:
  - Validating extracted values before processing
  - Handling ambiguous user input
  - Multi-step confirmation flows for high-stakes actions

**3. Research Resilient Currency Handling**
- Industry best practices for currency extraction in financial chatbots
- Patterns for explicit currency confirmation when value exceeds thresholds
- Safeguards to prevent silent defaults (e.g., never default to USD for high-value trades)

**4. Present Options to User**
- Document 2-3 solution approaches with trade-offs
- Include code examples from ADK docs where applicable
- **WAIT FOR USER CONFIRMATION** before implementing any solution

### Implementation Gate
⛔ **DO NOT IMPLEMENT Phase 10 fixes until:**
1. Research is complete and documented
2. Solution options are presented to user
3. User confirms preferred approach

## Acceptance Criteria

### AC1: No Silent Notification Failures
- [x] All `send_message()` calls MUST check the result
- [x] If `result.success == False`, log error with full context
- [x] Raise exception or trigger alert on notification failure
- [x] Never leave a request in "pending_compliance" without compliance being notified

### AC2: Guaranteed Notification Delivery (Outbox Pattern)
- [x] Create `notification_outbox` table to store pending notifications
- [x] Write notification to outbox in same DB transaction as status update
- [x] Background worker processes outbox and sends to Slack
- [x] Retry failed notifications with exponential backoff (max 5 attempts)
- [x] Alert on-call team if notification fails after all retries

### AC3: Fix Orchestrator Status Key
- [x] Add "status" key to error responses in `agent.py:100`
- [x] Add defensive check in handler to handle missing status gracefully
- [x] `test_auto_approve_flow` passes

### AC4: Add Missing Oracle FX Models
- [x] Define `OracleCurrency` and `OracleFx` SQLAlchemy models
- [x] Add to `models/__init__.py` exports
- [x] Add test data seeding in conftest.py
- [x] `test_checked_insider_checkbox_allows_request` passes

### AC5: Monitoring & Alerting
- [x] Dashboard shows count of pending notifications (should be ~0)
- [x] Dashboard shows count of failed notifications (ALERT if > 0)
- [x] Failed notifications visible in compliance dashboard for manual retry

## Out of Scope
- Changing Slack to a different messaging system
- Real-time websocket notifications
- Email fallback (future enhancement)

## Technical Approach

### Transactional Outbox Pattern
Instead of sending notifications directly after DB commit:
1. **Atomically** write notification to `notification_outbox` table in same transaction
2. **Background worker** polls outbox every 5 seconds
3. **Send** notification to Slack
4. **Mark** as sent only after successful delivery
5. **Retry** on failure with exponential backoff
6. **Alert** after max retries exhausted

This guarantees that if the DB transaction commits, the notification WILL eventually be delivered.

### References
- [Microservices.io: Transactional Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)
- [The outbox pattern in Python](https://blog.szymonmiks.pl/p/the-outbox-pattern-in-python/)
- [AWS Prescriptive Guidance: Transactional Outbox](https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html)

## Test Plan
1. **Unit tests**: All 730+ tests pass (including previously failing Phase 7 tests)
2. **Integration test**: Stop Slack mock → Submit request → Verify outbox entry → Start Slack mock → Verify notification sent
3. **Manual UAT**: Submit PAD request → Manager approve → Verify compliance channel notification
4. **Failure test**: Verify failed notifications are retried and eventually alert if exhausted

---

# Phase 10: CRITICAL AUTO-APPROVE THRESHOLD BYPASS

## Bug Report (2026-01-28)
A €262,900 trade was auto-approved when the threshold is €10,000.
- User entered: "EUR 262,900" (26x the threshold)
- System auto-approved as if it were a small trade
- Currency "EUR" was ignored, treated as USD or unspecified

## Root Cause Analysis (RESEARCH COMPLETE)

### Bug #1: Currency Never Extracted from User Input
**Severity: CRITICAL**

The `update_draft()` tool only accepts `estimated_value: float` - there is NO `currency` parameter.
When user types "EUR 262,900", the LLM can only extract the number (262900).
The currency "EUR" is completely ignored.

**Location**: `chatbot.py:201`
```python
async def update_draft(
    ...
    estimated_value: float | None = None,  # NO CURRENCY PARAMETER!
    ...
)
```

### Bug #2: Currency Defaults to USD
**Severity: CRITICAL**

`DraftRequest.currency` defaults to `"USD"` and is only updated from the security's `trade_currency`, never from user input.

**Location**: `session.py:27`
```python
class DraftRequest(BaseModel):
    currency: str = "USD"  # Default that overrides user intent
```

**Location**: `chatbot.py:315`
```python
"currency": selected_candidate.get('trade_currency', 'USD') or 'USD',
```

### Bug #3: value_gbp=None Defaults to LOW Risk
**Severity: HIGH**

If value is 0, falsy, or conversion fails, `value_gbp` is `None`.
The `assess_position_size()` returns LOW risk when `value_gbp is None`.
This means a missing value = auto-approve eligible!

**Location**: `risk_scoring.py:471-477`
```python
if value_gbp is None:
    return RiskFactor(
        name="Employee Position Size",
        level=FactorLevel.LOW,  # DANGEROUS DEFAULT!
        reason="No trade value provided - assuming low value",
    )
```

### Bug #4: Single MEDIUM Factor Allows Auto-Approve
**Severity: MEDIUM**

Auto-approve requires: HIGH factors = 0 AND MEDIUM factors ≥ 2.
With only 1 MEDIUM factor, auto-approve is still eligible.
Even a £200k trade (MEDIUM) can be auto-approved if no other factors flag.

**Location**: `risk_scoring.py:608-610`
```python
else:  # high_count == 0 and medium_count < 2
    overall_level = OverallRiskLevel.LOW
    approval_route = ApprovalRoute.AUTO_APPROVE  # Still auto-approve!
```

### Bug #5: DB Threshold Defaults Too High
**Severity: MEDIUM**

Code defaults differ from DB config defaults:
- Code: LOW < £50k, MEDIUM £50k-£100k, HIGH > £100k
- DB config: LOW < £100k, MEDIUM £100k-£1M, HIGH > £1M

**Location**: `risk_scoring_service.py:53-54`
```python
position_size_low_threshold=Decimal(str(thresholds.get("low_max", 100000))),
position_size_high_threshold=Decimal(str(thresholds.get("high_min", 1000000))),
```

## ADK Best Practices Research

### From Google ADK Documentation

1. **Action Confirmation Pattern**
   ADK provides `require_confirmation` for tools that perform high-stakes actions:
   ```python
   FunctionTool(submit_trade, require_confirmation=True)
   ```
   This pauses execution for user confirmation before proceeding.

2. **Conditional Confirmation**
   Confirmation can be conditional based on thresholds:
   ```python
   async def confirmation_threshold(amount: int, tool_context: ToolContext) -> bool:
       return amount > 10000  # Require confirmation for large amounts
   ```

3. **Structured Response Types**
   Function tools should return structured data with explicit fields:
   ```python
   def get_stock_price(symbol: str) -> dict:
       return {"price": 123.45, "currency": "USD", "symbol": symbol}
   ```

4. **Function Signature Best Practices**
   - Use explicit type hints for all parameters
   - Include clear docstrings explaining each parameter
   - Keep parameters simple (primitives preferred)
   - Make critical fields REQUIRED, not optional

### From Industry Best Practices

1. **Named Entity Recognition (NER)** for financial data extraction
   - Identify currencies, amounts, stock symbols explicitly
   - Never assume defaults for critical financial fields

2. **Confirmation & Escalation Patterns**
   - High-value transactions require explicit confirmation
   - Seamless escalation to human review for edge cases

3. **Real-Time Validation**
   - Validate extracted data against expected patterns
   - Echo back understanding before processing

### References
- [ADK Action Confirmations](https://google.github.io/adk-docs/tools-custom/confirmation/)
- [ADK Function Tools](https://google.github.io/adk-docs/tools-custom/function-tools/)
- [NLP in Trade Compliance](https://cleareye.ai/natural-language-processing-in-finance-trade-compliance/)
- [Finance AI Chatbots Best Practices](https://kaopiz.com/en/articles/finance-ai-chatbots/)

---

## ⛔ SOLUTION OPTIONS - AWAITING USER CONFIRMATION

### Option A: Structured Currency Extraction (Recommended)

**Approach**: Add explicit `currency` parameter to `update_draft` tool and require LLM to extract both value AND currency.

**Changes**:
1. Add `estimated_currency: str | None` parameter to `update_draft()` tool
2. Update system prompt to require currency extraction alongside value
3. Validate currency against known currency codes (EUR, USD, GBP, etc.)
4. Reject submission if currency is missing for values > £1,000

**Pros**:
- Clean, explicit data model
- LLM forced to extract currency
- Easy to validate

**Cons**:
- Requires prompt engineering
- LLM may still miss currency in edge cases

**Example tool signature**:
```python
async def update_draft(
    ...
    estimated_value: float | None = None,
    estimated_currency: str | None = None,  # NEW: EUR, USD, GBP
    ...
)
```

---

### Option B: Value Confirmation Pattern (Most Robust)

**Approach**: Before submission, echo back the extracted value/currency to user and require explicit confirmation.

**Changes**:
1. Add confirmation step after value extraction: "I understood €262,900 EUR - is that correct?"
2. Block submission until user confirms the amount
3. For values > threshold, show full confirmation with currency conversion to GBP

**Pros**:
- Zero-error guarantee (user sees exactly what system understood)
- Catches all extraction errors
- Follows ADK `require_confirmation` pattern

**Cons**:
- Additional step in UX flow
- Slightly slower submission process

**Example flow**:
```
User: "The value is EUR 262,900"
Bot: "I recorded: €262,900 EUR (~£225,000 GBP). Is this correct?"
User: "Yes"
Bot: "Thank you! Now, what's your justification..."
```

---

### Option C: Hybrid - Threshold-Based Confirmation

**Approach**: Combine structured extraction with confirmation for high-value trades only.

**Changes**:
1. Add `estimated_currency` parameter (Option A)
2. For values < £10,000: Accept without confirmation
3. For values ≥ £10,000: Require explicit confirmation with GBP conversion

**Pros**:
- Low friction for small trades (most common)
- High safety for large trades (where errors are costly)
- Best UX/safety balance

**Cons**:
- More complex implementation
- Two code paths to maintain

---

### Option D: Fix Auto-Approve Logic Only (Minimum Viable)

**Approach**: Keep currency extraction as-is, but fix the auto-approve logic bugs.

**Changes**:
1. **Never** return LOW risk when `value_gbp is None` - return MEDIUM instead
2. Change auto-approve to require `medium_count == 0` (not just < 2)
3. Align DB config defaults with code defaults

**Pros**:
- Smallest code change
- Quick fix

**Cons**:
- Does NOT fix the currency extraction bug
- User still won't see correct currency in confirmation
- Relies on fail-safe, not correct extraction

---

## Recommended Solution: Option C (Hybrid)

**Why**: It provides the best balance of:
- **Safety**: High-value trades get explicit confirmation
- **UX**: Low-value trades remain frictionless
- **Correctness**: Structured currency field enables validation
- **ADK Best Practice**: Follows confirmation pattern for high-stakes actions

**Implementation Steps**:
1. Add `estimated_currency` parameter to `update_draft()`
2. Update system prompt to extract currency
3. Add `confirm_trade_value()` step for values ≥ £10,000 GBP equivalent
4. Fix the defensive bugs (value_gbp=None should NOT default to LOW)
5. Align threshold configurations
6. Add comprehensive logging for auto-approve decisions

---

## ✅ PHASE 10 IMPLEMENTATION COMPLETE (2026-01-28)

**User confirmed Option C** with dynamic threshold from DB settings (`medium_value_threshold`).

### Implemented Fixes:

**Bug #3 - value_gbp=None defaults to MEDIUM** ✅
- File: `risk_scoring.py:471-477`
- Change: Returns `FactorLevel.MEDIUM` with reason "No trade value provided - requires manual review"
- Test: `test_phase10_auto_approve_fixes.py::TestBug3ValueGbpNone`

**Bug #4 - Single MEDIUM factor requires compliance** ✅
- File: `risk_scoring.py:604`
- Change: `medium_count >= 1` (was `>= 2`)
- Test: `test_phase10_auto_approve_fixes.py::TestBug4SingleMediumFactor`

**Bug #5 - Threshold defaults aligned** ✅
- File: `risk_scoring_service.py:53-54`
- Change: Default fallbacks now match code: £50k/£100k (was £100k/£1M)
- Test: `test_phase10_auto_approve_fixes.py::TestBug5ThresholdDefaults`

**Bug #1 & #2 - Currency extraction support** ✅
- File: `chatbot.py:201` - Added `estimated_currency` parameter to `update_draft()`
- File: `chatbot.py:248-253` - Validates and sets currency
- File: `chatbot.py:325-330` - Preserves user-provided currency over security's trade_currency
- Test: `test_phase10_auto_approve_fixes.py::TestCurrencyExtractionFields`

**Option C - High-value confirmation flow** ✅
- File: `session.py:61-65` - Added `pending_value_confirmation`, `value_confirmed`, `value_gbp_equivalent` fields
- File: `chatbot.py:206` - Added `confirm_value` parameter
- File: `chatbot.py:872-922` - Added `_check_high_value_confirmation()` method
- File: `chatbot.py:485-491` - Added `VALUE_CONFIRMATION_REQUIRED` hint state
- Test: `test_phase10_auto_approve_fixes.py::TestValueConfirmationFields`

### Updated Tests:
- `test_risk_scoring.py` - 3 tests updated to reflect new behavior
- `test_phase10_auto_approve_fixes.py` - 22 new tests for all Phase 10 fixes

### Test Results:
- All 80 related unit tests pass
- Original bug scenario (€262,900 auto-approved) now correctly requires SMF16 approval
