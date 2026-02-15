# Plan: Chatbot Tool Refactor Implementation

## Critical Constraint: Zero Functionality Loss

**This is a REFACTOR, not a rewrite.** Every single behavior in the current `update_draft` must be preserved or improved. No edge cases dropped.

### Current Functionality Audit (Must Preserve)

Before implementation, audit and document EVERY behavior in `update_draft`:

1. **Security Search Flow**
   - New search triggers DB lookup
   - Ambiguous results → DISAMBIGUATION_REQUIRED hint
   - No results → SECURITY_NOT_FOUND hint
   - User confirms by number (1, 2, 3)
   - User confirms by typing symbol
   - User confirms by saying "yes"
   - LLM recommendation tracking (`llm_recommended_index`)
   - Failed search tracking (`security_search_failed`, `failed_search_term`)

2. **Direction Handling**
   - Normalizes to uppercase BUY/SELL
   - Handles synonyms (purchase, dispose, etc.)

3. **Quantity Handling**
   - Extracts digits from strings ("15 lots" → 15)
   - Handles k notation (15k → 15000)

4. **Value/Currency Handling**
   - Validates against known currencies
   - Sets `currency_confirmed` flag
   - Triggers `pending_currency_confirmation` if missing
   - High-value confirmation flow (`pending_value_confirmation`)

5. **Justification Flow**
   - Quality assessment (WEAK/GOOD based on word count)
   - Coaching for weak justifications
   - `user_confirmed_weak_justification` flag

6. **Compliance Flags**
   - `has_inside_info` → `insider_info_confirmed`
   - `is_related_party` / `is_connected_person`
   - `existing_position`

7. **Derivative Detection**
   - `pending_derivative_question`
   - `pending_leveraged_question`
   - `derivative_justification`

8. **State Machine Hints**
   - DISAMBIGUATION_REQUIRED
   - SECURITY_NOT_FOUND
   - COACHING_REQUIRED
   - VALUE_CONFIRMATION_REQUIRED
   - CURRENCY_CONFIRMATION_REQUIRED
   - DRAFT_COMPLETE

---

## Phase 1: Comprehensive Test Suite (BEFORE any refactor)

### 1.1 Create Behavior Specification Tests

Create `tests/unit/test_chatbot_tool_behaviors.py` documenting EVERY behavior:

```python
class TestSecuritySearchBehaviors:
    """Document all security search behaviors - these MUST pass after refactor."""

    def test_new_search_triggers_db_lookup(self):
        """When search_term provided and no candidates, lookup is triggered."""

    def test_ambiguous_results_return_disambiguation_hint(self):
        """Multiple matches → instructional_hint='DISAMBIGUATION_REQUIRED'"""

    def test_user_confirms_by_number(self):
        """User typing '1' selects first candidate."""

    def test_user_confirms_by_symbol(self):
        """User typing 'AAPL' fuzzy matches to candidates."""

    def test_user_confirms_by_yes(self):
        """User typing 'yes' confirms LLM recommendation."""

    def test_no_results_return_not_found_hint(self):
        """Zero matches → instructional_hint='SECURITY_NOT_FOUND'"""

    def test_failed_search_tracked(self):
        """Failed searches set security_search_failed=True."""


class TestValueCurrencyBehaviors:
    """Document all value/currency behaviors."""

    def test_currency_extracted_and_validated(self):
        """Valid currency codes are accepted and stored."""

    def test_invalid_currency_rejected(self):
        """Invalid currency returns error hint."""

    def test_missing_currency_triggers_confirmation(self):
        """Value without currency → pending_currency_confirmation=True."""

    def test_high_value_triggers_confirmation(self):
        """Value > threshold → pending_value_confirmation=True."""

    def test_currency_confirmed_flag_set(self):
        """Valid currency sets currency_confirmed=True."""


class TestJustificationBehaviors:
    """Document justification quality flow."""

    def test_short_justification_marked_weak(self):
        """< 5 words → justification_quality='WEAK'"""

    def test_weak_justification_triggers_coaching(self):
        """WEAK quality → instructional_hint='COACHING_REQUIRED'"""

    def test_user_can_confirm_weak_justification(self):
        """user_confirmed_weak_justification=True bypasses coaching."""


class TestComplianceFlagBehaviors:
    """Document compliance flag handling."""

    def test_has_inside_info_false_sets_confirmed(self):
        """has_inside_info=False → insider_info_confirmed=True."""

    def test_has_inside_info_true_blocks_request(self):
        """has_inside_info=True triggers compliance escalation."""

    def test_connected_person_maps_to_related_party(self):
        """is_connected_person sets is_related_party."""


class TestDerivativeBehaviors:
    """Document derivative detection flow."""

    def test_derivative_triggers_question(self):
        """Derivative inst_type → pending_derivative_question=True."""

    def test_leveraged_triggers_question(self):
        """Leveraged product → pending_leveraged_question=True."""


class TestStateHintBehaviors:
    """Document all instructional hints."""

    def test_all_fields_complete_returns_draft_complete(self):
        """All required fields → instructional_hint='DRAFT_COMPLETE'"""

    def test_missing_security_returns_needs_security(self):
        """No security → hint indicates security needed."""
```

### 1.2 Create End-to-End Conversation Tests

```python
class TestFullConversationFlows:
    """Test complete conversation scenarios."""

    async def test_happy_path_buy_equity(self):
        """User buys 100 AAPL shares for $15,000 USD."""

    async def test_happy_path_sell_with_eur(self):
        """User sells BUND for EUR 262,900."""

    async def test_ambiguous_security_disambiguation(self):
        """User searches 'apple', gets options, selects by number."""

    async def test_weak_justification_coaching(self):
        """User gives weak justification, gets coached, confirms."""

    async def test_high_value_confirmation_flow(self):
        """High value trade triggers confirmation."""

    async def test_currency_confirmation_when_missing(self):
        """Value without currency triggers confirmation prompt."""

    async def test_derivative_detection_flow(self):
        """Derivative product triggers derivative questions."""
```

### 1.3 Run Full Test Suite and Record Baseline

```bash
# Record current behavior
pytest tests/unit/test_chatbot*.py -v --tb=short > baseline_results.txt
pytest tests/integration/test_chatbot*.py -v --tb=short >> baseline_results.txt
```

---

## Phase 2: Tool Extraction (One at a Time)

### Strategy: Extract and Verify

For each tool:
1. Extract logic from `update_draft` into new tool
2. Have `update_draft` delegate to new tool
3. Run full test suite - must pass
4. Repeat for next tool

### 2.1 Extract `set_trade_value` (Already Done - Phase 11)

Status: COMPLETE

### 2.2 Extract `set_security`

```python
async def set_security(
    self,
    user_id: str,
    thread_ts: str,
    search_term: str,
) -> dict[str, Any]:
    """Search for a security. Extract ONLY the base symbol.

    NEVER include: calls, puts, options, futures, lots, shares, @, prices

    Returns:
        - success: True if search completed
        - candidates: List of matching securities (if multiple)
        - selected: The selected security (if unambiguous or confirmed)
        - instructional_hint: Next action (DISAMBIGUATION_REQUIRED, SECURITY_NOT_FOUND, etc.)
    """
```

**Logic to extract from update_draft:**
- Lines 300-430: Security confirmation/lookup logic
- Lines 430-500: Database search logic
- Lines 500-550: Candidate selection logic

### 2.3 Extract `set_direction`

```python
async def set_direction(
    self,
    user_id: str,
    thread_ts: str,
    direction: str,
) -> dict[str, Any]:
    """Set trade direction.

    Args:
        direction: BUY or SELL (case insensitive, synonyms accepted)
    """
```

**Logic to extract:**
- Lines 237-238: Direction normalization

### 2.4 Extract `set_quantity`

```python
async def set_quantity(
    self,
    user_id: str,
    thread_ts: str,
    quantity: int,
) -> dict[str, Any]:
    """Set trade quantity.

    Args:
        quantity: Number of shares/lots/units
    """
```

**Logic to extract:**
- Lines 239-249: Quantity extraction and sanitization

### 2.5 Extract `set_justification`

```python
async def set_justification(
    self,
    user_id: str,
    thread_ts: str,
    justification: str,
) -> dict[str, Any]:
    """Set trade justification.

    Returns quality assessment and coaching hint if needed.
    """
```

**Logic to extract:**
- Lines 250-256: Justification and quality assessment
- Coaching logic from hint generation

### 2.6 Extract `set_compliance_flags`

```python
async def set_compliance_flags(
    self,
    user_id: str,
    thread_ts: str,
    has_inside_info: bool | None = None,
    is_related_party: bool | None = None,
    existing_position: bool | None = None,
) -> dict[str, Any]:
    """Set compliance flags.

    At least one flag must be provided per call.
    """
```

**Logic to extract:**
- Lines 282-291: Compliance flag handling

### 2.7 Extract `confirm_selection`

```python
async def confirm_selection(
    self,
    user_id: str,
    thread_ts: str,
    selection: str | int,
) -> dict[str, Any]:
    """Confirm security selection from candidates.

    Args:
        selection: Index (1-based), symbol, or "yes" for recommendation
    """
```

**Logic to extract:**
- Lines 304-330: Selection confirmation logic

---

## Phase 3: Update System Prompt

### 3.1 Simplify Prompt

Move ALL extraction rules to tool docstrings. Prompt becomes:

```python
SYSTEM_PROMPT = """You are the PA Dealing Assistant helping employees submit trade requests.

WORKFLOW:
1. Greet user warmly
2. Ask for ONE missing field at a time
3. Call the appropriate tool after each answer
4. Follow the tool's instructional_hint for next action
5. When hint is DRAFT_COMPLETE, call show_preview

TOOLS:
- set_security(search_term): Search for securities
- set_direction(direction): Set BUY or SELL
- set_quantity(quantity): Set number of shares/lots
- set_trade_value(amount, currency): Set value AND currency (both required)
- set_justification(justification): Set trade reason
- set_compliance_flags(...): Set insider info, related party flags
- confirm_selection(selection): Confirm security from options
- show_preview(): Show final summary

Be conversational. Never list all questions at once.
"""
```

### 3.2 Register All Tools

```python
def get_agent(self) -> Agent:
    if self._agent is None:
        self._agent = Agent(
            name="pa_dealing_chatbot",
            model=get_model(),
            description="Conversational assistant for PA Dealing requests.",
            instruction=SYSTEM_PROMPT,
            tools=[
                FunctionTool(self.set_security),
                FunctionTool(self.set_direction),
                FunctionTool(self.set_quantity),
                FunctionTool(self.set_trade_value),
                FunctionTool(self.set_justification),
                FunctionTool(self.set_compliance_flags),
                FunctionTool(self.confirm_selection),
                FunctionTool(self.get_active_draft_state),
                FunctionTool(self.show_preview),
            ],
        )
    return self._agent
```

---

## Phase 4: Deprecate update_draft

### 4.1 Keep as Fallback (Initially)

```python
async def update_draft(self, ...) -> dict[str, Any]:
    """DEPRECATED: Use specific tools instead.

    This method delegates to the appropriate focused tool.
    Will be removed in future version.
    """
    logger.warning("update_draft called - consider using specific tools")
    # Delegate to appropriate tool based on which params are set
    ...
```

### 4.2 Remove After Verification

Once all tests pass with new tools, remove `update_draft` entirely.

---

## Phase 5: Final Verification

### 5.1 Run Full Test Suite

```bash
# All existing tests must pass
pytest tests/unit/test_chatbot*.py -v
pytest tests/integration/test_chatbot*.py -v
pytest tests/unit/test_phase10*.py -v

# New behavior tests must pass
pytest tests/unit/test_chatbot_tool_behaviors.py -v
```

### 5.2 Manual UAT

Test each conversation flow manually:
1. Simple buy equity (happy path)
2. Sell with EUR currency
3. Ambiguous security search
4. Weak justification coaching
5. High-value confirmation
6. Derivative detection

### 5.3 Compare Baseline

```bash
# Compare results to baseline
pytest tests/ -v --tb=short > refactor_results.txt
diff baseline_results.txt refactor_results.txt
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Lost edge case | Comprehensive behavior tests BEFORE refactor |
| LLM uses wrong tool | Clear tool names and docstrings |
| State inconsistency | All tools use same SessionManager |
| Test gaps | Manual UAT for each flow |

---

## Files to Modify

1. `src/pa_dealing/agents/slack/chatbot.py`
   - Add 6 new tool methods
   - Update get_agent() to register tools
   - Simplify SYSTEM_PROMPT
   - Deprecate then remove update_draft

2. `tests/unit/test_chatbot_tool_behaviors.py` (NEW)
   - Comprehensive behavior specification tests

3. `tests/unit/test_chatbot_tools.py` (NEW)
   - Unit tests for each new tool

4. `tests/integration/test_chatbot_refactor.py` (NEW)
   - E2E conversation flow tests

---

## Definition of Done

- [x] All 100+ existing chatbot tests pass
- [x] New behavior specification tests pass
- [x] New E2E conversation tests pass
- [x] Manual UAT for 6 conversation flows
- [x] No functionality regression
- [x] System prompt < 500 tokens
- [x] Each tool has < 50 lines of code
- [x] Code review approved
