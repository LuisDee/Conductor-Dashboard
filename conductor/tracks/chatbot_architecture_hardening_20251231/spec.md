# Track Spec: Chatbot Architecture Hardening

## Problem Statement
The current Slack Chatbot implementation relies on the LLM's context window to maintain the state of an unsubmitted "draft" request (ticker, quantity, direction). It also relies on the LLM to format the summary UI as a JSON block.

This leads to two critical risks:
1.  **Hallucination:** The LLM might "forget" or hallucinate a value (e.g., changing quantity from 100 to 1000) when passing arguments to the final submission tool.
2.  **Formatting Errors:** The LLM might generate invalid JSON or inconsistent formatting for the summary block, causing UI rendering failures.

## Goal
Refactor the Chatbot to use a **Stateful, Code-Driven** architecture.
1.  **Server-Side State:** User inputs are saved to a Python `SessionManager`. Subsequent steps (compliance check, submission) read from this trusted state, not from the LLM's memory.
2.  **Code-Driven UI:** The `preview_request` tool generates the Block Kit UI deterministically using Python code, removing the need for the LLM to output JSON.

## Core Components

### 1. Session Manager
A robust in-memory (or Redis-ready) store for `DraftRequest` objects key by `user_id`.

```python
class DraftRequest(BaseModel):
    security_identifier: str | None
    direction: str | None
    quantity: int | None
    justification: str | None
    ...
```

### 2. Stateful Tools
Tools will be refactored to be "Context-Aware" rather than "Stateless".

*   `identify_security(search_term)` -> Finds security AND saves it to `session.draft.security`.
*   `set_trade_details(quantity, direction, ...)` -> Validates AND saves to `session.draft`.
*   `check_compliance(user_id)` -> Reads from `session.draft`.
*   `submit_current_draft(user_id)` -> Submits `session.draft`.

### 3. Code-Driven Preview
*   `preview_request(user_id)` -> Reads `session.draft`, builds Block Kit UI (using `ui.py`), posts to Slack, and returns "Preview sent" to LLM.

## Success Criteria
- [x] Submitting a request relies on ZERO parameters passed from LLM memory to the final `submit` function (except `user_id`).
- [x] The `[SUMMARY]` JSON prompt instruction is removed entirely.
- [x] The Chatbot uses the exact same `ui.py` components as the Manager Notifications for consistent rendering.
