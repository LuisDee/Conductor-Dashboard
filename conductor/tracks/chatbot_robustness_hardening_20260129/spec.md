# Track: Chatbot Robustness Hardening

**Goal:** Eliminate "hallucinated re-asking" and "data overwrite" bugs in the PA Dealing chatbot by hardening the prompt context, enforcing state immutability, and dynamically scoping tools.

**Root Cause:**
The LLM receives a generic "continue" signal after UI interactions without context of what just changed. It ignores the updated `[GROUND TRUTH]` state and follows a hallucinated script, leading to:
1.  **Redundant Questioning:** Re-asking "Is this a leveraged product?" after it was answered via button.
2.  **State Overwrite:** Overwriting valid justifications with compliance answers (e.g., "No inside info").

## Implementation Phases

### Phase 1: Context & Prompt Hardening (High Impact)

**Objective:** Ensure the LLM explicitly knows *what* just happened and *what* it should do next, preventing the "standard script" hallucination.

#### 1.1 Replace Bare "continue" with Action Summary
**File:** `src/pa_dealing/agents/slack/handlers.py`
**Method:** `_continue_chatbot_flow`

Instead of sending `"continue"`, send a system instruction detailing the state change.

*   **Logic:**
    1.  Load the current draft.
    2.  Calculate missing fields (import `_get_missing_fields_static` from `chatbot.py`).
    3.  Construct a prompt:
        ```text
        [SYSTEM] The user just completed a UI interaction. The draft has been updated.
        Do NOT re-ask any field already present in [GROUND TRUTH].
        Fields still missing: {missing_fields}.
        Next action: Ask for the next missing field from the list above, ONE at a time.
        ```
    4.  Call `process_message` with this prompt.

#### 1.2 Harden `SYSTEM_PROMPT`
**File:** `src/pa_dealing/agents/slack/chatbot.py`
**Constant:** `SYSTEM_PROMPT`

Add strict negative constraints to the system prompt:
*   "NEVER ask about a field that already has a value in [GROUND TRUTH]."
*   "NEVER call set_justification when the user is answering a Yes/No compliance question."
*   "If a tool call returns REJECTED, read the reason and do NOT retry."

---

### Phase 2: State Locking & Guards (Safety Net)

**Objective:** Make completed fields immutable to preventing accidental overwrites by the LLM.

#### 2.1 Add Field Lock Mechanism
**File:** `src/pa_dealing/agents/slack/session.py`
**Model:** `DraftRequest`

*   Add `locked_fields: set[str] = Field(default_factory=set)` to the model.
*   Ensure this field is persisted in the database.

#### 2.2 Implement Field Locking Logic
**Locations:**
*   **`src/pa_dealing/agents/slack/handlers.py`**:
    *   In `_handle_derivative_question`: Add "is_derivative" to `locked_fields`.
    *   In `_handle_leveraged_question`: Add "is_leveraged" to `locked_fields`.
*   **`src/pa_dealing/agents/slack/chatbot.py`**:
    *   In `set_justification`: If quality is "GOOD", add "justification" to `locked_fields`.
    *   In `set_compliance_flags`: Add "has_inside_info" and "is_related_party" to `locked_fields`.
    *   In `set_security`: After confirmation, add "security" to `locked_fields`.

#### 2.3 Implement Tool Guards
**File:** `src/pa_dealing/agents/slack/chatbot.py`
**Methods:** All `set_*` tools (`set_justification`, `set_compliance_flags`, etc.)

*   **Logic:** At the start of each tool, check if the corresponding field is in `draft.locked_fields`.
*   **Action:** If locked, return a structured rejection:
    ```json
    {
        "status": "rejected",
        "reason": "REJECTED: '{field}' is locked. Move on to the next missing field.",
        "instructional_hint": "PROCEED: {field} is already set. Ask for the next missing field."
    }
    ```
*   **Specific Guard for `set_justification`:**
    *   Even if not locked, reject updates that contain compliance keywords ("inside info", "related party") or are too short (< 5 words) if the current quality is already "GOOD".

---

### Phase 3: Dynamic Tool Scoping (Defense in Depth)

**Objective:** Physically prevent the LLM from calling tools for fields that are already completed.

#### 3.1 Define Tool Scoping Rules
**File:** `src/pa_dealing/agents/slack/chatbot.py`

Create a mapping of tools to their required completion fields:
```python
FIELD_TOOL_SCOPING = {
    "set_security": ["security_identifier"],
    "set_justification": ["justification"], # Only if quality is GOOD
    "set_compliance_flags": ["insider_info_confirmed", "is_related_party"],
    # ...
}
```

#### 3.2 Implement Tool Filter
**File:** `src/pa_dealing/agents/slack/chatbot.py`
**Method:** `get_available_tools` (new method)

*   Input: `DraftRequest`, `all_tools` list.
*   Logic: Iterate through `FIELD_TOOL_SCOPING`. If the fields for a tool are present (and valid/locked), exclude that tool from the list.
*   Output: Filtered list of tools.

#### 3.3 Apply Scoping in `process_message`
**File:** `src/pa_dealing/agents/slack/chatbot.py`
**Method:** `process_message`

*   Call `get_available_tools` before invoking the LLM.
*   Pass the filtered list to the LLM generation call.

---

## Verification Strategy

**Test Case 1: Leverage Re-ask (Bug 1)**
1.  Start a new request.
2.  Provide security, quantity, justification.
3.  Click "No" on "Is this a leveraged product?".
4.  **Expect:** Bot acknowledges ("Thanks!") and IMMEDIATELY asks for "Inside Information" (or next missing field).
5.  **Fail:** Bot asks "Is this a leveraged product?" again.

**Test Case 2: Justification Overwrite (Bug 2)**
1.  Start a new request.
2.  Provide a GOOD justification (> 5 words).
3.  When asked for Inside Information, reply text: "No inside information".
4.  **Expect:** Bot accepts compliance flag. Justification remains unchanged.
5.  **Fail:** Bot says "I noticed your justification is quite brief...".

## Constraints
*   Do NOT revert commit `d82e04b` (structured logging).
*   Ensure `DraftRequest` changes are backward compatible (default factory handles this).
*   Keep architecture (state-driven prompting) intact.
