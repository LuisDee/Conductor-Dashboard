# Plan: Chatbot Architecture Hardening (COMPLETED)

## Phase 1: Session Infrastructure (COMPLETED)
- [x] **Task: Create Session Manager & Models**
    - [x] Create `src/pa_dealing/agents/slack/session.py`.
    - [x] Define `DraftRequest` Pydantic model with all required fields.
    - [x] Implement `SessionManager` class with `get_draft(user_id)`, `update_draft(user_id, data)`, `clear_draft(user_id)`.
    - [x] Integrate `SessionManager` into `SlackSocketHandler` (replacing the generic `InMemorySessionService` usage for draft data).

## Phase 2: Stateful Tool Refactoring (COMPLETED)
- [x] **Task: Refactor "Setter" Tools**
    - [x] Update `lookup_security` to `identify_security(user_id, search_term)`. It must save the result to the session draft.
    - [x] Create/Update `set_trade_details(user_id, ...)` to save direction, quantity, justification, etc., to the session draft.
- [x] **Task: Refactor "Getter" Tools**
    - [x] Update `check_compliance(user_id)` to read *only* from the session draft. Remove trade arg parameters.
    - [x] Update `submit_pad_request` to `submit_current_draft(user_id)`. It must read all data from the session draft.

## Phase 3: Code-Driven UI & Formatting (COMPLETED)
- [x] **Task: Implement `preview_request` Tool**
    - [x] Create `preview_request(user_id)` tool in `chatbot.py`.
    - [x] Implement logic to read the draft, call `ui.build_trade_summary_blocks`, and post to Slack via `SlackClient`.
    - [x] Ensure `ui.build_trade_summary_blocks` handles missing fields gracefully (draft state).
- [x] **Task: Update Chatbot System Prompt**
    - [x] Remove all instructions about `[SUMMARY]` and JSON formatting.
    - [x] Update instructions to use the new "Stateful" workflow (Call Identify -> Call Set Details -> Call Preview -> Call Submit).

## Phase 4: Integration & Verification (COMPLETED)
- [x] **Task: Integration**
    - [x] Wire up the new tools in `chatbot.py`.
    - [x] Ensure the `SlackSocketHandler` initializes the session correctly on new DMs.
- [x] **Task: Verification**
    - [x] Manually test the full flow: "Buy 100 BARC" -> "Yes" -> Submit.
    - [x] Verify persistence: "Buy 100 BARC", then "Actually make it 200". Verify `check_compliance` sees 200.
    - [x] Verify `submit_current_draft` uses the stored data.
