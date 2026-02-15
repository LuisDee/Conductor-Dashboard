# Spec: Risk Assessment Fixes & Derivative Justification

## Type
Bug Fix + Enhancement

## Overview
Address multiple bugs and enhancements in the PA Dealing risk assessment and Slack conversation flow:
1. Bot ignores user clarifying questions and advances conversation state
2. Risk scoring incorrectly classifies derivative/leveraged trades as LOW risk and auto-approves them
3. Add mandatory derivative justification capture with new database field
4. Update risk scoring thresholds and factor levels
5. Add holding period risk factor based on executed trade history

## Bug 1: Bot Skips User Questions in Conversation Flow

### Current Behavior
When a user types a question (e.g., "is that LC?") during instrument confirmation, the bot treats it as a response and advances the conversation state, skipping the confirmation step entirely.

### Expected Behavior
The bot should detect interrogative patterns in user messages (e.g., ends with "?", contains "is that", "what is", "which", "does", etc.) and respond with a clarifying answer instead of advancing the flow. The user must explicitly confirm (button click or affirmative text like "yes") to proceed.

### Acceptance Criteria
- Messages containing question marks or interrogative patterns are detected
- Bot responds helpfully to the question without changing conversation state
- Conversation only advances on explicit confirmation

## Bug 2: Derivative/Leveraged Risk Scoring & Auto-Approval

### Current Behavior
- A leveraged derivative trade was classified as LOW risk and AUTO APPROVED
- Leveraged products should trigger an immediate recommendation to reject (advisory)
- Derivative products should be HIGH risk, not LOW

### Expected Behavior
- **Leveraged products**: "Strongly Advise to Reject" advisory — should NEVER be auto-approved
- **Derivative products (non-leveraged)**: HIGH risk level — routes to SMF16 escalation, never auto-approved
- Neither leveraged nor derivative trades should ever be auto-approved regardless of other factors

### Acceptance Criteria
- Leveraged trade → "Strongly Advise to Reject" advisory, no auto-approval
- Derivative (non-leveraged) trade → HIGH risk, no auto-approval
- Auto-approval logic explicitly checks `is_derivative` and `is_leveraged` flags

## Enhancement 1: Derivative Justification Field

### Description
When a product is identified as a derivative and the user confirms it is NOT leveraged, the Slack conversation flow must:
1. Inform the user: "You're trading a derivative, are you sure?"
2. User confirms
3. Ask: "Please provide your justification for trading this derivative"
4. Capture freeform text response

### Data Model
- New field: `derivative_justification` (Text, nullable) on `pad_request`
- Only populated for derivative products
- Requires Alembic migration
- Propagate through: Slack flow → API schemas → Dashboard request detail → Compliance notifications

### Acceptance Criteria
- Derivative flow includes confirmation + justification capture (2-step gate)
- `derivative_justification` stored in database
- Visible in dashboard request detail view
- Included in compliance/manager notifications when present
- Field is optional (null) for non-derivative trades

## Enhancement 2: Updated Risk Scoring Thresholds

### Value Thresholds (Position Size)
- **LOW**: < £50,000 (was £100,000)
- **MEDIUM**: £50,000 – £100,000 (was £100,000 – £1,000,000)
- **HIGH**: > £100,000 (was > £1,000,000)

### Connected Person
- Change from **HIGH** to **MEDIUM** risk
- Connected person alone should no longer escalate to SMF16

### Acceptance Criteria
- Config values updated for position size thresholds
- Connected person factor returns MEDIUM instead of HIGH
- All existing risk scoring tests updated to reflect new thresholds

## Enhancement 3: Holding Period Risk Factor

### Description
Add a new risk factor: if the user wants to SELL an instrument they have previously BOUGHT (executed, not just approved) within a configurable window (default: 30 days), flag as HIGH risk.

### Detection Logic
- Query `pad_request` joined with `pad_execution`
- Filter: same `employee_id`, same instrument (by `isin` or `security_id`), `direction = 'BUY'`, `status = 'executed'`, `pad_execution.executed_at` within the configured window
- Only applies to SELL actions

### Configuration
- `holding_period_days`: default 30, configurable

### Acceptance Criteria
- SELL request for recently-executed BUY (within 30 days) → HIGH risk
- SELL request with no recent executed BUY → LOW risk (this factor)
- Only executed trades count (approved-but-not-executed are ignored)
- Window is configurable
- New factor appears in risk breakdown

## Out of Scope
- Changes to the dashboard configuration UI for these thresholds (separate track)
- Firm trading conflict detection (separate track)
- Text-based yes/no response parsing (separate track already exists)
