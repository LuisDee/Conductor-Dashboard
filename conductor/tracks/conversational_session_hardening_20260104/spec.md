# Conversational Session Hardening Specification

## 1. Overview
This feature hardens the Chatbot's session management by enforcing Thread-Based State Isolation. The goal is to prevent "sticky sessions" where new requests in the main DM accidentally resume old, unrelated drafts. It also introduces "Justification Coaching" to improve compliance quality.

## 2. Technical Architecture

### 2.1 Session Management
- **Key Strategy:** `thread_ts` as the primary isolation key.
- **Main DM (No Thread):**
    - `thread_ts` is `None`.
    - Treated as a "Launcher" context.
    - Never stores state. Always triggers new intent detection.
- **Thread Context:**
    - `thread_ts` is set (e.g., `1767515610.656599`).
    - Stores the `DraftRequest` state.
    - `session_id` in DB: `slack_thread_<thread_ts>`.

### 2.2 Chatbot Logic (`process_message`)
- **Main DM Flow:**
    1. Receive message.
    2. Detect intent (LLM).
    3. Create new Thread.
    4. Save Draft keyed to new Thread ID.
    5. Post response in new Thread.
- **Thread Flow:**
    1. Receive message in thread.
    2. Load Draft using `thread_ts`.
    3. Process update.
    4. Save Draft.

### 2.3 Justification Coaching
- **Schema Update:** Add `justification_quality` enum to `IntentUpdates`.
- **Values:** `GOOD`, `WEAK`, `MISSING`.
- **Logic:**
    - If `WEAK` and `!user_confirmed`: Ask "Too brief. Add more?".
    - If `WEAK` and `user_confirmed`: Accept.

## 3. Data Model Changes
### `DraftRequest` Schema
- Add `justification_quality: str | None`
- Add `user_confirmed_weak_justification: bool`

## 4. Testing Strategy
- **Unit:** Test `SessionManager` generates correct keys.
- **E2E:** Mock Slack events to verify:
    - Main DM -> Spawns Thread.
    - Thread A -> Updates Draft A.
    - Thread B -> Updates Draft B.
    - Draft A is unaffected by B.
