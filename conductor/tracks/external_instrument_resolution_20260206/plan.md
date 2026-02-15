# Implementation Plan: External Instrument Resolution Layer

## Phase 1: Infrastructure & Abstraction
- [ ] Create `src/pa_dealing/instruments/external_resolver.py` defining the `ExternalInstrumentResolver` protocol and `ResolvedInstrument` dataclass.
- [ ] Implement `MockExternalResolver` for TDD.
- [ ] Set up test fixtures in `tests/fixtures/external_resolver/` for EODHD.
- [ ] Implement `EODHDResolver` using `httpx`.
- [ ] Add config for `EODHD_API_TOKEN` (defaulting to the provided key in `.env.example`) and `EXTERNAL_RESOLVER_PROVIDER`.

## Phase 2: Repository Integration (TDD)
- [ ] Write integration tests for the 4-outcome logic in `repository.py`.
- [ ] Modify `_search_instruments` in `src/pa_dealing/db/repository.py` to include the external resolution tier.
- [ ] Implement exact-match lookup in Bloomberg table using resolved ISIN/Ticker/Exchange.
- [ ] Ensure graceful fallthrough on external API failure.

## Phase 3: Orchestrator & Risk Scoring
- [ ] Update `RiskScoringResult` and `AdvisoryResult` to include the lookup outcome (1-4).
- [ ] Implement "Outcome 4" detection: if no match anywhere, trigger a high-risk compliance flag.
- [ ] Update `OrchestratorAgent` to pass the resolution outcome to the `PADRequest` record.
- [ ] Add Alembic migration to store `resolution_outcome` and `external_provider_metadata` in `pad_request` table.

## Phase 4: Chatbot UX
- [ ] Implement Outcome 4 clarification prompt: "I couldn't identify [XYZ]... proceed with flag?"
- [ ] Update `PADealingChatbot` to handle the "user proceeds anyway" path by setting the high-risk flag.
- [ ] Verify the re-run logic if a user provides a better identifier (ISIN/Ticker) in response to the prompt.

## Phase 5: Audit & Monitoring
- [ ] Implement detailed logging of external API calls (query, response, outcome).
- [ ] Add health checks for external API connectivity.

## Phase 6: Final Verification
- [ ] ✅ Final check: Ensure the implementation matches all requirements in `gemini-plan-eodhd.plan`.
- [ ] Perform full regression test of the instrument lookup pipeline.

## Future Work (Deferred)
- [ ] Phase 2 Rollout: OpenFIGI implementation (Pending legal sign-off).
- [ ] Implement `OpenFIGIResolver`.
- [ ] Switch provider via config and verify full derivative coverage.
