# Implementation Plan: Instrument Identity Refactor

## Phase 1: TDD - Test Definition (RED)
- [x] Update `tests/test_instrument_lookup.py` to use `search_instruments` and `resolve_instrument_identity`.
- [x] Verify tests fail (Red state).

## Phase 2: Repository Implementation (GREEN)
- [x] Rename `lookup_instrument` to `_search_instruments` in `src/pa_dealing/db/repository.py`.
- [x] Implement `async def search_instruments(session, term) -> InstrumentLookupResult`.
- [x] Rename `lookup_instrument_comprehensive` to `resolve_instrument_identity` in `src/pa_dealing/db/repository.py`.
- [x] Update internal calls in `resolve_instrument_identity` to use `_search_instruments`.
- [x] Run `pytest tests/test_instrument_lookup.py` to verify pass (Green state).

## Phase 3: Layer Updates & Refactor (REFACTOR)
- [x] Update `src/pa_dealing/services/pad_service.py` to use new names.
- [x] Update `src/pa_dealing/agents/database/agent.py` tool definitions.
- [x] Update `src/pa_dealing/agents/orchestrator/agent.py` calls to use `resolve_instrument_identity`.
- [x] Update `src/pa_dealing/agents/slack/chatbot.py` calls to use `search_instruments`.
- [x] Update `src/pa_dealing/api/routes/dashboard.py` calls.
- [x] Verify all tests pass after each update.

## Phase 4: Test Suite Alignment (Regression)
- [x] Update `tests/integration/test_security_confirmation_flow.py`.
- [x] Update `tests/integration/test_slack_mock.py` mocks.

## Phase 5: Documentation & Final Verification
- [x] Update `docs/tooling/instrument-lookup.md`.
- [x] Verify the Gemini Skill `instrument-lookup` is accurate.
- [x] Run full test suite: `.venv/bin/pytest tests/test_instrument_lookup.py`.