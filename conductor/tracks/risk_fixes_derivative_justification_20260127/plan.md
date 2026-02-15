# Plan: Risk Assessment Fixes & Derivative Justification

## Phase 1: Question Detection in Slack Conversation Flow

- [x]Task 1.1: Write Tests — Question detection utility
  - [x]Create unit tests for `detect_question()` helper function
  - [x]Test patterns: trailing "?", "is that", "what is", "which", "does this", "can you", etc.
  - [x]Test negative cases: "yes", "no", affirmative statements
  - [x]Test edge cases: "yes?", mixed content

- [x]Task 1.2: Implement — Question detection utility
  - [x]Add `detect_question()` function in chatbot.py (or conversation utils)
  - [x]Return True for interrogative patterns, False otherwise

- [x]Task 1.3: Write Tests — Conversation flow does not advance on questions
  - [x]Test that when user sends a question during instrument confirmation, state does NOT change
  - [x]Test that bot responds with clarifying info (e.g., repeats the instrument details)
  - [x]Test that explicit confirmation ("yes", button click) still advances correctly

- [x]Task 1.4: Implement — Integrate question detection into conversation flow
  - [x]In the instrument confirmation handler, check for questions before advancing
  - [x]If question detected, respond with instrument details and re-prompt for confirmation
  - [x]Ensure button-based confirmation still works unchanged

- [x]Task 1.5: Lint, format, verify in Docker

- [x]Task 1.6: Commit Phase 1

- [x]Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: Fix Derivative/Leveraged Risk Scoring & Auto-Approval

- [x]Task 2.1: Write Tests — Derivative = HIGH risk
  - [x]Test that `is_derivative=True` (non-leveraged) produces HIGH risk level
  - [x]Test that derivative trades are NOT auto-approve eligible
  - [x]Test that derivative trades route to SMF16 escalation

- [x]Task 2.2: Write Tests — Leveraged = Strongly Advise to Reject
  - [x]Test that `is_leveraged=True` produces "Strongly Advise to Reject" advisory
  - [x]Test that leveraged trades are NOT auto-approve eligible
  - [x]Test that leveraged trades are never auto-approved regardless of other factors

- [x]Task 2.3: Implement — Update risk scoring for derivatives
  - [x]Modify `assess_instrument_type()` in `risk_scoring.py` so derivative = HIGH (not NA/advisory)
  - [x]Ensure derivative HIGH factor flows through aggregation correctly

- [x]Task 2.4: Implement — Update auto-approval guard for derivative/leveraged
  - [x]In `compliance_decision_service.py`, add explicit checks: if `is_derivative` or `is_leveraged`, block auto-approval
  - [x]Leveraged products: set advisory flag "Strongly Advise to Reject"

- [x]Task 2.5: Update existing tests to reflect new behavior

- [x]Task 2.6: Lint, format, verify in Docker

- [x]Task 2.7: Commit Phase 2

- [x]Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: Derivative Justification Field & Slack Flow

- [x]Task 3.1: Create Alembic migration
  - [x]Add `derivative_justification` (Text, nullable) to `pad_request` table

- [x]Task 3.2: Update SQLAlchemy model
  - [x]Add `derivative_justification` field to `PADRequest` in `pad.py`

- [x]Task 3.3: Update API schemas
  - [x]Add `derivative_justification` to Pydantic request/response schemas

- [x]Task 3.4: Write Tests — Slack conversation flow for derivative justification
  - [x]Test: derivative detected + not leveraged → "you're trading a derivative, are you sure?" prompt
  - [x]Test: user confirms → "please provide justification" prompt
  - [x]Test: user provides justification → stored in session state → proceeds to declaration
  - [x]Test: non-derivative trades skip justification flow entirely

- [x]Task 3.5: Implement — Slack conversation flow for derivative justification
  - [x]After leveraged=No confirmation, add derivative confirmation step ("you're trading a derivative, are you sure?")
  - [x]After derivative confirmation, prompt for freeform justification text
  - [x]Store `derivative_justification` in session state
  - [x]Pass through to `pad_request` on submission

- [x]Task 3.6: Write Tests — Dashboard displays derivative justification
  - [x]Test request detail view shows `derivative_justification` when present
  - [x]Test it is hidden/absent for non-derivative requests

- [x]Task 3.7: Implement — Dashboard request detail
  - [x]Add `derivative_justification` to request detail view
  - [x]Only render when value is non-null

- [x]Task 3.8: Implement — Include in compliance/manager notifications
  - [x]Add `derivative_justification` to Slack notification blocks when present

- [x]Task 3.9: Lint, format, verify in Docker, rebuild dashboard

- [x]Task 3.10: Commit Phase 3

- [x]Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: Updated Risk Scoring Thresholds & Factor Levels

- [x]Task 4.1: Write Tests — New value thresholds
  - [x]Test position size < £50k → LOW
  - [x]Test position size £50k–£100k → MEDIUM
  - [x]Test position size > £100k → HIGH

- [x]Task 4.2: Write Tests — Connected person = MEDIUM
  - [x]Test connected person → MEDIUM (not HIGH)
  - [x]Test connected person alone does NOT escalate to SMF16

- [x]Task 4.3: Implement — Update config thresholds
  - [x]`position_size_low_threshold` = 50,000
  - [x]`position_size_high_threshold` = 100,000

- [x]Task 4.4: Implement — Connected person factor → MEDIUM
  - [x]Modify `assess_connected_person()` in `risk_scoring.py` to return MEDIUM instead of HIGH

- [x]Task 4.5: Update all affected existing tests

- [x]Task 4.6: Lint, format, verify in Docker

- [x]Task 4.7: Commit Phase 4

- [x]Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)

## Phase 5: Holding Period Risk Factor

- [x]Task 5.1: Write Tests — Holding period detection
  - [x]Test: SELL request + executed BUY within 30 days for same instrument → HIGH
  - [x]Test: SELL request + executed BUY older than 30 days → LOW (this factor)
  - [x]Test: SELL request + approved-but-not-executed BUY within 30 days → LOW (ignored)
  - [x]Test: BUY request → LOW (factor only applies to SELL)
  - [x]Test: configurable window (e.g., 14 days, 60 days)

- [x]Task 5.2: Implement — Holding period check query
  - [x]Create function to query `pad_request` joined with `pad_execution`
  - [x]Filter: same employee, same instrument (ISIN/security_id), direction=BUY, status=executed, executed_at within window
  - [x]Return boolean + execution date if found

- [x]Task 5.3: Implement — New risk factor `assess_holding_period()`
  - [x]Add to `risk_scoring.py` as Factor 7
  - [x]HIGH if recent executed BUY found and current action is SELL
  - [x]LOW otherwise

- [x]Task 5.4: Implement — Add configurable `holding_period_days` to config
  - [x]Default: 30 days
  - [x]Add to `config.py`

- [x]Task 5.5: Integrate into risk scoring pipeline
  - [x]Call `assess_holding_period()` in the scoring service
  - [x]Include in factor breakdown output

- [x]Task 5.6: Update existing aggregation tests if factor count changed

- [x]Task 5.7: Lint, format, verify in Docker

- [x]Task 5.8: Commit Phase 5

- [x]Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)

## Phase 6: Full Regression Testing

- [x]Task 6.1: Run full backend test suite in Docker
- [x]Task 6.2: Run Playwright E2E tests (4+ workers)
- [x]Task 6.3: Fix any regressions found
- [x]Task 6.4: Final commit

- [x]Task: Conductor - User Manual Verification 'Phase 6' (Protocol in workflow.md)
