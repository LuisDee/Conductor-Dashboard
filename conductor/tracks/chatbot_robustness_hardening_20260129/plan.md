# Plan: Chatbot Architecture Hardening

**Goal:** Eliminate "hallucinated re-asking" and "data overwrite" bugs in the PA Dealing chatbot by hardening the prompt context, enforcing state immutability, and dynamically scoping tools.

**Root Cause:**
The LLM receives a generic "continue" signal after UI interactions without context of what just changed. It ignores the updated `[GROUND TRUTH]` state and follows a hallucinated script, leading to:
1.  **Redundant Questioning:** Re-asking "Is this a leveraged product?" after it was answered via button.
2.  **State Overwrite:** Overwriting valid justifications with compliance answers (e.g., "No inside info").

---

## Implementation Phases

### Phase 1: Context & Prompt Hardening (COMPLETE)

- [x] **1.1 Replace Bare "continue" with Action Summary**
  - **Logic:** Sent descriptive `[SYSTEM]` message in `_continue_chatbot_flow` with missing fields.
- [x] **1.2 Harden `SYSTEM_PROMPT`**
  - **Logic:** Added strict negative rules to `CRITICAL RULES`.

---

### Phase 2: State Locking & Guards (COMPLETE)

- [x] **2.1 Add Field Lock Mechanism**
  - **Logic:** Added `locked_fields: list[str]` to `DraftRequest`.
- [x] **2.2 Implement Field Locking Logic**
  - **Logic:** Implemented locking in `handlers.py` (buttons) and `chatbot.py` (tools).
- [x] **2.3 Implement Tool Guards**
  - **Logic:** All `set_*` tools now check `locked_fields` and return `REJECTED` status. Added specific "suspicious overwrite" guard for `set_justification`.

---

### Phase 3: Dynamic Tool Scoping (COMPLETE)

- [x] **3.1 Define Tool Scoping Rules**
- [x] **3.2 Implement Tool Filter (`get_available_tools`)**
- [x] **3.3 Apply Scoping in `process_message`**
  - **Logic:** AI now only sees tools for fields that aren't yet completed and locked.

---

### Phase 4: Verification (OUTSTANDING)

- [ ] **4.1 Live LLM Verification**
  - Task: Test the exact flow once LiteLLM quota resets to ensure prompt adherence and tool scoping work as intended.
  - Test Cases:
    1. Click "No" on leveraged product -> Verify no re-ask.
    2. Answer "No inside info" via text -> Verify justification remains unchanged.

---

## Verification Strategy (Ready for Test)

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
