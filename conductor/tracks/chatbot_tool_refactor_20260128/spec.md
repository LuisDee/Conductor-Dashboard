# Spec: Chatbot Tool Refactor - Split Monolithic update_draft into Focused Tools

## Problem Statement

The current PA Dealing chatbot uses a single monolithic `update_draft()` tool with **16+ optional parameters**. This design causes the LLM to inconsistently extract data because:

1. **ADK Schema Behavior**: Google ADK marks parameters with `default=None` as OPTIONAL in the tool schema
2. **LLM Behavior**: When parameters are optional, the LLM frequently skips extraction - it "satisfies" the tool call with minimal params
3. **Silent Data Loss**: Critical fields like currency get silently defaulted instead of extracted

### Evidence from Phase 11 (2026-01-28)

A EUR 262,900 trade was stored as USD 262,900 because:
- `estimated_currency` parameter had `default=None` (OPTIONAL)
- LLM only extracted `estimated_value=262900`
- Currency defaulted to USD silently

**Fix that worked**: Created `set_trade_value(amount, currency)` with NO defaults - both REQUIRED.

### Evidence from Quantity Bug (2026-01-28)

A trade for 2,000 units was recorded as 12,000 units because:
- `quantity` parameter had `default=None` (OPTIONAL)
- Multiple numbers in message: "2,000 units", "131.45", "262,900"
- LLM hallucinated quantity without clear extraction instruction

**Fix needed**: Create `set_quantity(quantity: int)` with NO default - REQUIRED.

### Current Monolithic Tool Signature

```python
async def update_draft(
    self,
    user_id: str,
    thread_ts: str,
    direction: str | None = None,           # OPTIONAL
    quantity: int | None = None,            # OPTIONAL
    security_search_term: str | None = None, # OPTIONAL
    justification: str | None = None,        # OPTIONAL
    justification_quality: str | None = None, # OPTIONAL
    estimated_value: float | None = None,    # OPTIONAL
    estimated_currency: str | None = None,   # OPTIONAL - PROBLEM!
    confirm_currency_usd: bool | None = None, # OPTIONAL
    confirm_value: bool | None = None,       # OPTIONAL
    has_inside_info: bool | None = None,     # OPTIONAL
    is_related_party: bool | None = None,    # OPTIONAL
    is_connected_person: bool | None = None, # OPTIONAL
    existing_position: bool | None = None,   # OPTIONAL
    user_confirmed_weak_justification: bool | None = None, # OPTIONAL
) -> dict[str, Any]:
```

**Problem**: LLM sees 14 optional fields and cherry-picks which to fill.

---

## Proposed Solution: Focused Single-Purpose Tools

Split into **9 focused tools**, each with REQUIRED parameters for their domain:

### 1. `set_security(search_term)` - REQUIRED
```python
async def set_security(
    user_id: str,
    thread_ts: str,
    search_term: str,  # REQUIRED - no default
) -> dict[str, Any]:
    """Search for a security. Extract ONLY the base symbol.

    NEVER include: calls, puts, options, futures, lots, shares, @, prices

    EXAMPLES:
    - "buying 5 lots of bund calls @ 100" -> search_term="bund"
    - "sell 10 shares of AAPL" -> search_term="aapl"
    - "trade FGBL futures" -> search_term="fgbl"
    """
```

### 2. `set_direction(direction)` - REQUIRED
```python
async def set_direction(
    user_id: str,
    thread_ts: str,
    direction: str,  # REQUIRED - "BUY" or "SELL"
) -> dict[str, Any]:
    """Set trade direction. Must be BUY or SELL.

    EXAMPLES:
    - "I want to buy" -> direction="BUY"
    - "selling my position" -> direction="SELL"
    - "purchase" -> direction="BUY"
    - "dispose of" -> direction="SELL"
    """
```

### 3. `set_quantity(quantity)` - REQUIRED
```python
async def set_quantity(
    user_id: str,
    thread_ts: str,
    quantity: int,  # REQUIRED - numeric only
) -> dict[str, Any]:
    """Set trade quantity. Extract numeric value only.

    CRITICAL: Remove commas when extracting!
    Quantity is the NUMBER OF UNITS, NOT the price or total value.

    EXAMPLES:
    - "2,000 units" -> quantity=2000 (remove comma!)
    - "buy 1,500 shares" -> quantity=1500
    - "5 lots" -> quantity=5
    - "100 shares" -> quantity=100
    - "15k" -> quantity=15000

    DO NOT confuse with:
    - Price per unit (e.g., 131.45)
    - Total value (e.g., 262,900)
    """
```

### 4. `set_trade_value(amount, currency)` - BOTH REQUIRED (Already implemented in Phase 11)
```python
async def set_trade_value(
    user_id: str,
    thread_ts: str,
    amount: float,   # REQUIRED
    currency: str,   # REQUIRED
) -> dict[str, Any]:
    """Set trade value AND currency. BOTH REQUIRED.

    Currency mapping: EUR, $->USD, GBP, CHF, JPY

    EXAMPLES:
    - "EUR 262,900" -> amount=262900, currency="EUR"
    - "$500k" -> amount=500000, currency="USD"
    - "100,000 pounds" -> amount=100000, currency="GBP"
    """
```

### 5. `set_justification(justification)` - REQUIRED
```python
async def set_justification(
    user_id: str,
    thread_ts: str,
    justification: str,  # REQUIRED
) -> dict[str, Any]:
    """Set trade justification. Capture the user's full reasoning.

    Returns quality assessment (WEAK/GOOD) in response.
    If WEAK, instructional_hint will prompt for more detail.
    """
```

### 6. `set_compliance_flags(...)` - All flags REQUIRED when called
```python
async def set_compliance_flags(
    user_id: str,
    thread_ts: str,
    has_inside_info: bool,      # REQUIRED
    is_related_party: bool,     # REQUIRED
) -> dict[str, Any]:
    """Set compliance flags. Both flags REQUIRED when called.

    EXAMPLES:
    - "No inside info, not related" -> has_inside_info=False, is_related_party=False
    - "Yes I have inside info" -> has_inside_info=True, is_related_party=<ask>
    """
```

### 7. `confirm_selection(response)` - For security confirmation/disambiguation
```python
async def confirm_selection(
    user_id: str,
    thread_ts: str,
    response: str,  # REQUIRED - user's raw input ("yes", "1", "BUND", etc.)
) -> dict[str, Any]:
    """Confirm security selection from search results.

    CRITICAL: Pass user input EXACTLY as typed.

    When user types a NUMBER ("1", "2", "3"):
    - Pass LITERALLY: response="1"
    - DO NOT interpret as the ticker name!
    - WRONG: User types "1" -> response="AAPL CT"
    - RIGHT: User types "1" -> response="1"

    When user CONFIRMS ("yes", "correct", "yep"):
    - Pass as typed: response="yes"
    - System uses recommended match (index 0)

    When user types SYMBOL ("BUND", "AAPL"):
    - Pass as typed: response="BUND"
    - System fuzzy-matches to candidates

    EXAMPLES:
    - User: "1" -> response="1"
    - User: "yes" -> response="yes"
    - User: "BUND" -> response="BUND"
    - User: "the first one" -> response="1"
    """
```

### 8. `confirm_pending(...)` - For confirmation flows
```python
async def confirm_pending(
    user_id: str,
    thread_ts: str,
    confirmation_type: str,  # REQUIRED: "currency_usd", "value", "weak_justification"
    confirmed: bool,         # REQUIRED: True if user confirms
) -> dict[str, Any]:
    """Handle confirmation responses for pending states.

    Use when:
    - CURRENCY_CONFIRMATION_REQUIRED: User confirms USD
    - VALUE_CONFIRMATION_REQUIRED: User confirms high-value amount
    - COACHING_REQUIRED: User proceeds despite weak justification

    EXAMPLES:
    - User confirms "yes, USD" -> confirmation_type="currency_usd", confirmed=True
    - User confirms high value -> confirmation_type="value", confirmed=True
    - User says "proceed" after coaching -> confirmation_type="weak_justification", confirmed=True
    """
```

### 9. `set_value_pending_currency(amount)` - When currency unknown
```python
async def set_value_pending_currency(
    user_id: str,
    thread_ts: str,
    amount: float,  # REQUIRED
) -> dict[str, Any]:
    """Set trade value when currency is not yet known.

    Use when user provides value WITHOUT currency (e.g., "the value is 262,900").
    System will set pending_currency_confirmation=True and ask for currency.

    After user specifies currency, call set_trade_value with both amount and currency.

    EXAMPLE:
    - User: "the trade is worth 50,000" -> amount=50000
    - Returns: instructional_hint="CURRENCY_CONFIRMATION_REQUIRED"
    """
```

---

## Simplified System Prompt

```python
SYSTEM_PROMPT = """You are the PA Dealing Assistant helping employees submit trade requests.

WORKFLOW:
1. Greet user, ask for ONE missing field at a time
2. After each answer, call the appropriate tool
3. Check the tool's 'instructional_hint' for next action
4. When DRAFT_COMPLETE, call show_preview

CRITICAL RULES:
- NEVER list all questions at once - ask ONE at a time
- NEVER suggest securities from your training data - ONLY use search results
- ALWAYS call a tool after user provides information
- When presenting search results, show ONLY the top match for confirmation
- When user types a NUMBER for selection ("1", "2"), pass it LITERALLY to confirm_selection

TOOLS AVAILABLE:
- set_security: Search for securities by name/ticker
- confirm_selection: Confirm or select from search results (pass user input literally)
- set_direction: Buy or sell
- set_quantity: Number of shares/lots
- set_trade_value: Set amount AND currency together (both required)
- set_value_pending_currency: Set amount when currency unknown
- set_justification: Reason for trade
- set_compliance_flags: Inside info, related party checks
- confirm_pending: Confirm currency/value/coaching prompts
- show_preview: Final summary

Be conversational. Follow the instructional_hint from each tool response.
"""
```

---

## Benefits

| Before (Monolithic) | After (Focused Tools) |
|---------------------|----------------------|
| 16 optional params | 9 tools with required params |
| LLM skips fields | LLM must provide all params |
| Complex docstring | Simple, focused docstrings |
| Silent defaults | Explicit errors on missing data |
| Hard to debug | Clear tool call traces |
| 200+ line function | ~30 lines per tool |

### ADK Best Practice Alignment (From Official Documentation)

From Google ADK documentation (https://google.github.io/adk-docs/tools-custom/function-tools/):

#### 1. Required vs Optional Parameters
> "A parameter is considered **required** if it has a type hint but **no default value**. The LLM must provide a value for this argument when it calls the tool."

> "A parameter is considered **optional** if you provide a **default value**."

**Our approach**: All critical fields (amount, currency, search_term, direction) have NO defaults = REQUIRED.

#### 2. Best Practices (Direct Quotes)
> "**Fewer Parameters are Better**: Minimize the number of parameters to reduce complexity."

> "**Simple Data Types**: Favor primitive data types like `str` and `int` over custom classes whenever possible."

> "**Meaningful Names**: The function's name and parameter names significantly influence how the LLM interprets and utilizes the tool. Choose names that clearly reflect the function's purpose."

**Our approach**: Split 16-param monolithic tool into 9 focused tools with 1-3 params each.

#### 3. Docstrings Are Critical
> "The docstring of your function serves as the tool's **description** and is sent to the LLM. Therefore, a well-written and comprehensive docstring is crucial for the LLM to understand how to use the tool effectively."

**Our approach**: Each tool has detailed docstring with extraction examples (moved from system prompt).

#### 4. Return Type Best Practice
> "The preferred return type for a Function Tool is a **dictionary**... As a best practice, include a **'status'** key in your return dictionary to indicate the overall outcome (e.g., 'success', 'error', 'pending')."

**Our approach**: All tools return `{"status": "success", "instructional_hint": "...", ...}` pattern.

---

## Acceptance Criteria

### AC1: Tool Separation
- [ ] `update_draft` replaced with 9 focused tools
- [ ] Each tool has only REQUIRED parameters (no defaults for critical fields)
- [ ] Each tool returns `instructional_hint` for conversation flow

### AC2: Extraction Accuracy
- [ ] Currency always extracted when value provided
- [ ] Security search term always cleaned of derivatives/quantities
- [ ] Direction always normalized to BUY/SELL

### AC2b: Quantity Extraction
- [ ] "2,000 units" correctly extracted as quantity=2000
- [ ] Quantity not confused with price or total value
- [ ] Commas in quantity properly handled

### AC3: Backward Compatibility
- [ ] All existing test scenarios pass
- [ ] Conversation flow unchanged from user perspective
- [ ] Draft state structure unchanged

### AC4: Simplified Prompt
- [ ] System prompt under 500 tokens
- [ ] No extraction rules in prompt (moved to tool docstrings)
- [ ] Clear tool list with one-line descriptions

### AC5: Confirmation Flows
- [ ] Currency confirmation ("yes, USD") handled by `confirm_pending`
- [ ] Value confirmation for high-value trades works via `confirm_pending`
- [ ] "proceed" after weak justification coaching works via `confirm_pending`
- [ ] Value-without-currency triggers CURRENCY_CONFIRMATION_REQUIRED via `set_value_pending_currency`

### AC6: Selection Handling
- [ ] "yes" confirmation works via `confirm_selection(response="yes")`
- [ ] Numeric selection "1" works via `confirm_selection(response="1")`
- [ ] Text selection "BUND" works via `confirm_selection(response="BUND")`
- [ ] Number NOT interpreted (user types "1" → "1", NOT "AAPL")

### AC7: Side Effects Preserved
- [ ] `_check_existing_position()` called after security confirmed
- [ ] `_check_high_value_confirmation()` called after value set
- [ ] Currency preserved when security selected (don't overwrite user's EUR)
- [ ] Derivative auto-detection preserved from inst_type

---

## Out of Scope

- Changing the DraftRequest model structure
- Modifying the orchestrator/risk scoring
- Adding new compliance checks
- UI changes

---

## Gap Resolutions (From Pre-Implementation Analysis)

These gaps were identified during analysis and addressed in this spec:

| # | Gap | Risk | Resolution |
|---|-----|------|------------|
| 1 | Confirmation actions missing | HIGH | Added `confirm_pending` tool (Section 8) |
| 2 | `confirm_selection` integer-only | CRITICAL | Changed to `response: str` parameter |
| 3 | Currency fallback orphaned | HIGH | Added `set_value_pending_currency` tool (Section 9) |
| 4 | Missing compliance params | LOW | Acceptable - `existing_position` auto-populated |
| 5 | Side effects not replicated | HIGH | AC7 ensures side effects preserved in each tool |
| 6 | Security logic split | MEDIUM | Careful extraction with shared utilities |
| 7 | Prompt critical instructions | CRITICAL | Preserved in simplified prompt (financial advice, number literal rules) |
| 8 | State machine ownership | MEDIUM | All tools call shared `determine_instructional_hint()` |
| 9 | Multi-field extraction | MEDIUM | ADK supports parallel tool calls - documented in prompt |

### Implementation Notes

1. **Shared Hint Generation**: Extract `determine_instructional_hint(draft)` from `update_draft` lines 590-617 into a reusable function. Every tool must call this after updating draft state.

2. **Side Effect Helper**: Create `_run_post_update_side_effects(user_id, thread_ts, changed_fields)` that conditionally runs:
   - `_check_existing_position()` if ticker was set
   - `_check_high_value_confirmation()` if value was set

3. **Currency Preservation**: `confirm_selection` must check `draft.currency_confirmed` before setting currency from security's `trade_currency`.

4. **Parallel Tool Calls**: ADK supports parallel tool calls. When user provides multiple pieces of information in one message, LLM should call ALL relevant tools.

---

## Test Plan

1. **Unit Tests**: Each tool tested in isolation
2. **Integration Tests**: Full conversation flow with mocked LLM
3. **Extraction Tests**: Verify currency, quantity, direction extraction accuracy
4. **Regression Tests**: All existing chatbot tests pass

---

## References

- Phase 11 implementation: `set_trade_value` with required params
- ADK Function Tools: https://google.github.io/adk-docs/tools-custom/function-tools/
- ADK Best Practices: Required params force complete extraction
