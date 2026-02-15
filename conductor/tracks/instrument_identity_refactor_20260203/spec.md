# Specification: Instrument Identity Refactor

## 1. Problem Statement
The current instrument lookup functions (`lookup_instrument` and `lookup_instrument_comprehensive`) have ambiguous names that cause confusion between "fuzzy searching" (returning many results) and "identity resolution" (returning one authoritative result). This "leaky abstraction" makes the backend logic harder to reason about and maintain.

## 2. Goals
- Clearly separate human-facing search logic from system-facing resolution logic.
- Align the codebase with Pythonic API design best practices.
- Improve code readability and reduce the risk of identity drift when saving trades.

## 3. Proposed Changes

### Core Repository (`src/pa_dealing/db/repository.py`)
- **Rename** `lookup_instrument` to `_search_instruments` (private).
- **Create** `search_instruments` (public): A wrapper around `_search_instruments` that returns an `InstrumentLookupResult` (list of ranked matches).
- **Rename** `lookup_instrument_comprehensive` to `resolve_instrument_identity` (public): A function that returns a single `InstrumentInfo` or `None`.

### Service Layer (`src/pa_dealing/services/pad_service.py`)
- Update `PADService` to expose `search_instruments` and `resolve_instrument_identity`.

### Agent Tools
- Update `DatabaseAgent` (`src/pa_dealing/agents/database/agent.py`) to expose `search_instruments` as a tool.
- Update `OrchestratorAgent` (`src/pa_dealing/agents/orchestrator/agent.py`) to use `resolve_instrument_identity` when locking in trade details.

## 4. Verification Plan
- **Unit Tests**: Update `tests/test_instrument_lookup.py` to use the new names and verify return types.
- **Integration Tests**: Verify the Slack Chatbot and Dashboard still function correctly with the renamed functions.
- **Documentation**: Update `docs/tooling/instrument-lookup.md` to reflect the new architecture.
