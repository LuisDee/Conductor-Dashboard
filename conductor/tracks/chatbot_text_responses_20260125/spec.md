# Spec: Chatbot Text Response Handling

## Problem Statement

Users are shown button-based questions for derivative and leveraged product classification in the Slack chatbot. Currently, only button clicks are handled. When users type "yes" or "no" instead of clicking buttons, they receive no response from the bot. This creates UX friction and confusion, as the natural conversational flow is interrupted.

The chatbot should accept both button clicks AND text responses for these questions, providing a seamless conversational experience regardless of how users choose to respond.

## User Stories

1. **As a user**, when I'm asked a yes/no question in the chatbot, I want to type my answer naturally (like "yes" or "no") instead of being forced to use buttons.

2. **As a user**, when I type "yes" or "no" to answer a derivative/leveraged question, I expect the conversation to continue just as it would if I clicked a button.

3. **As a user**, I expect case-insensitive parsing so that "YES", "Yes", "yes", and "y" all work the same way.

4. **As a user**, if I type an ambiguous response (like "maybe" or "idk"), I want helpful guidance on what responses are accepted.

## Current Behavior

**Derivative Question Flow:**
1. Bot asks: "Is this a derivative product?"
2. User sees two buttons: "Yes" and "No"
3. If user clicks button → conversation continues
4. If user types "yes" → message is sent to LLM, which doesn't understand the context

**Leveraged Question Flow:**
1. Bot asks: "Is this a leveraged product?"
2. User sees two buttons: "Yes" and "No"
3. If user clicks button → conversation continues
4. If user types "yes" → message is sent to LLM, which doesn't understand the context

## Proposed Solution

### 1. Text Response Parsing Logic

Add deterministic text parsing in `chatbot.py` **before** sending messages to the LLM. When a derivative or leveraged question is pending, intercept text responses and handle them identically to button clicks.

**Parse logic:**
```python
def parse_yes_no_response(text: str) -> Optional[bool]:
    """
    Parse text response as yes/no.
    Returns:
        True for yes, False for no, None for ambiguous/invalid
    """
    normalized = text.strip().lower()

    # Explicit yes responses
    if normalized in ["yes", "y", "yeah", "yep", "yup", "true"]:
        return True

    # Explicit no responses
    if normalized in ["no", "n", "nope", "nah", "false"]:
        return False

    # Ambiguous or invalid
    return None
```

### 2. Integration Points

**In `chatbot.py:handle_user_message()`:**
```python
# Before LLM processing
if draft.pending_derivative_question:
    parsed = parse_yes_no_response(message)
    if parsed is not None:
        # Mirror button handler logic
        draft.is_derivative = parsed
        draft.pending_derivative_question = False
        await self._continue_after_derivative_question(draft)
        return
    else:
        # Ambiguous response - prompt user
        await self.client.send_message(
            channel_id=channel_id,
            text="Please answer 'yes' or 'no' (or click a button above)."
        )
        return

if draft.pending_leveraged_question:
    parsed = parse_yes_no_response(message)
    if parsed is not None:
        # Mirror button handler logic
        draft.is_leveraged = parsed
        draft.pending_leveraged_question = False
        await self._continue_after_leveraged_question(draft)
        return
    else:
        # Ambiguous response - prompt user
        await self.client.send_message(
            channel_id=channel_id,
            text="Please answer 'yes' or 'no' (or click a button above)."
        )
        return
```

### 3. Flow Continuation

The `_continue_after_derivative_question()` and `_continue_after_leveraged_question()` methods already exist and handle the logic for what happens after answering these questions. Text responses will reuse these exact same methods, ensuring identical behavior.

**Flow after derivative question:**
- If no → Continue to next field (often leveraged question)
- If yes → Continue to next field (often leveraged question)

**Flow after leveraged question:**
- If no → Continue to field collection
- If yes → Continue to field collection

### 4. Backward Compatibility

This change is 100% backward compatible:
- Button handlers remain unchanged
- Existing button-based flows continue to work
- Text parsing only activates when pending_derivative_question or pending_leveraged_question flags are True
- No changes to button rendering or button handler logic

## Requirements

### Functional Requirements

1. **FR1**: Parse "yes", "y", "yeah", "yep", "yup" (case-insensitive) as affirmative responses
2. **FR2**: Parse "no", "n", "nope", "nah" (case-insensitive) as negative responses
3. **FR3**: Update `draft.is_derivative` based on text response when `pending_derivative_question` is True
4. **FR4**: Update `draft.is_leveraged` based on text response when `pending_leveraged_question` is True
5. **FR5**: Clear pending question flags after successful text response
6. **FR6**: Continue conversation flow identically to button click handler
7. **FR7**: Handle ambiguous responses (e.g., "maybe", "idk") with helpful error message
8. **FR8**: Preserve all existing button-based functionality

### Non-Functional Requirements

1. **NFR1**: Text parsing must complete in <100ms (deterministic, no LLM call)
2. **NFR2**: Zero regression in existing button-based tests
3. **NFR3**: Code maintainability - text and button handlers share the same continuation logic
4. **NFR4**: Clear logging for debugging (log when text response is parsed)

## Implementation Plan

### Phase 1: Core Parsing Logic
1. Add `parse_yes_no_response()` helper function in `chatbot.py`
2. Add unit tests for parsing logic (test various yes/no variations)

### Phase 2: Integration with Derivative Question
1. Add text parsing check in `handle_user_message()` for `pending_derivative_question`
2. Mirror button handler logic for state updates
3. Add integration test: user types "yes" to derivative question
4. Add integration test: user types "no" to derivative question
5. Add integration test: user types "maybe" to derivative question (ambiguous)

### Phase 3: Integration with Leveraged Question
1. Add text parsing check in `handle_user_message()` for `pending_leveraged_question`
2. Mirror button handler logic for state updates
3. Add integration test: user types "yes" to leveraged question
4. Add integration test: user types "no" to leveraged question

### Phase 4: Edge Cases & Validation
1. Test case-insensitivity ("YES", "Yes", "yes")
2. Test variations ("yeah", "yup", "nope", "nah")
3. Test whitespace handling (" yes ", "  no  ")
4. Verify all existing button tests still pass

## Files to Modify

1. **`src/pa_dealing/agents/slack/chatbot.py`**
   - Add `parse_yes_no_response()` helper
   - Add text parsing logic in `handle_user_message()` for derivative question
   - Add text parsing logic in `handle_user_message()` for leveraged question
   - Add ambiguous response handling

2. **`tests/unit/test_chatbot_text_responses.py`** (NEW)
   - Test `parse_yes_no_response()` with various inputs
   - Test derivative question text response flow
   - Test leveraged question text response flow
   - Test ambiguous response handling
   - Test case-insensitivity
   - Test whitespace handling

3. **`tests/integration/test_chatbot_flow.py`** (MODIFY)
   - Add end-to-end test: user types "yes" to both questions
   - Add end-to-end test: user types "no" to both questions
   - Add end-to-end test: user types "yes" to derivative, "no" to leveraged
   - Verify no regression in existing button-based tests

## Success Criteria

### Acceptance Criteria

- [ ] Users can type "yes", "y", "yeah", "yep", "yup" (case-insensitive) for affirmative responses
- [ ] Users can type "no", "n", "nope", "nah" (case-insensitive) for negative responses
- [ ] Conversation proceeds to next question or field collection after text response
- [ ] Ambiguous responses (e.g., "maybe") receive helpful error message
- [ ] All existing button-based tests continue to pass (zero regression)
- [ ] Text parsing completes in <100ms (no LLM overhead)
- [ ] Clear logs indicate when text response was parsed

### Test Coverage

- [ ] Unit tests for `parse_yes_no_response()` (20+ test cases)
- [ ] Integration tests for derivative question text response (3+ test cases)
- [ ] Integration tests for leveraged question text response (3+ test cases)
- [ ] End-to-end test for full conversation flow with text responses
- [ ] Regression validation: all existing chatbot tests pass

## Example Conversation Flow

### Before (Button-Only)

```
Bot: Is this a derivative product?
     [Yes] [No]

User: yes
Bot: <no response - message sent to LLM, which doesn't understand>
```

### After (Button + Text)

```
Bot: Is this a derivative product?
     [Yes] [No]

User: yes
Bot: Is this a leveraged product?
     [Yes] [No]

User: no
Bot: Great! Let's collect some details about your trade...
```

## Edge Cases

| Input | Parsed Result | Action |
|-------|---------------|--------|
| "yes" | True | Continue flow |
| "YES" | True | Continue flow |
| "y" | True | Continue flow |
| "yeah" | True | Continue flow |
| "no" | False | Continue flow |
| "NO" | False | Continue flow |
| "n" | False | Continue flow |
| "nope" | False | Continue flow |
| " yes " | True | Continue flow (whitespace trimmed) |
| "maybe" | None | Show error: "Please answer 'yes' or 'no'" |
| "idk" | None | Show error: "Please answer 'yes' or 'no'" |
| "what?" | None | Show error: "Please answer 'yes' or 'no'" |

## Risks & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Text parsing conflicts with LLM processing | Low | Medium | Parse BEFORE LLM processing; clear precedence |
| User types partial sentence starting with "yes" | Low | Low | Accept only exact matches (after trim/lowercase) |
| Ambiguous responses frustrate users | Medium | Low | Clear error message with accepted values |
| Regression in button-based flow | Low | High | Run full test suite; preserve all button handlers |

## Future Enhancements (Out of Scope)

- Support for other languages ("sí", "oui", "ja")
- Fuzzy matching for typos ("yse", "noo")
- Support for full sentences ("Yes, it is a derivative")
- Conversational rephrasing ("Could you clarify if...")
