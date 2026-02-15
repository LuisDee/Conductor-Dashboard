# Spec: External Instrument Resolution Layer

## Overview
Add an external instrument resolution layer (EODHD initially) as a safety net to identify securities that fail internal matching. This prevents false auto-approvals for real securities that Mako trades but couldn't be resolved internally due to typos or alias differences. **OpenFIGI migration is currently on hold as future work.**

## Current State
- **Lookup**: 3-tier internal lookup (Bloomberg, Mappings, Product) + Fuzzy Fallback.
- **Risk**: If no match is found, the system currently proceeds with the raw input. If risk is low, it might auto-approve.
- **Vulnerability**: A real security we trade might be typed with a typo, fail the internal lookup, and be auto-approved because it's "unknown".

## Proposed Architecture: 4-Outcome Routing
The new flow calls the external resolver FIRST to get canonical identifiers (ISIN/Ticker/Exchange), then matches those against our internal database.

### The 4 Outcomes
1. **External Match + Internal Match**: Resolved externally, found in our tradeable list. -> **Insider Trading Checks**.
2. **External Match + No Internal Match**: Resolved externally, but we DON'T trade it. -> **Confident Auto-Approve**.
3. **External Fail + Internal Match**: External lookup failed/missed, but our internal tiers (including fuzzy) found it. -> **Insider Trading Checks**.
4. **Nothing Matches Anywhere**: Completely unknown. -> **Clarification Prompt + Manual Review Flag** (NEVER auto-approve).

## Design Details

### 1. Abstraction Layer
- **`ExternalInstrumentResolver` Interface**: A protocol defining `resolve(query: str) -> list[ResolvedInstrument]`.
- **`ResolvedInstrument` Model**: Standardized output containing `name`, `ticker`, `isin`, `sedol`, `exchange`, `security_type`, `source_provider`.
- **Provider Switch**: Config-driven (`EXTERNAL_RESOLVER_PROVIDER=eodhd|openfigi`).

### 2. Implementation: EODHD (Phase 1)
- **Endpoint**: `GET https://eodhd.com/api/search/{query}`
- **Auth**: `api_token` query parameter.
- **Format**: `fmt=json`.
- **Key**: `688c6b8c5ed5a0.06847867` (to be stored in `.env`).
- **Coverage**: Stocks, ETFs, Bonds. Catch equity typos immediately.

### 3. Integration Points
- **Repository**: Modify `_search_instruments` in `src/pa_dealing/db/repository.py` to call the external resolver as Tier 0.
- **Orchestrator**: Update `process_pad_request` to handle Outcome 4 (flagging high risk).
- **Chatbot**: Update `PADealingChatbot` to handle the clarification/proceed flow for Outcome 4.

### 4. Safety & Compliance
- **Graceful Degradation**: If external API is down/slow, fall back to internal tiers. Do NOT block or auto-approve.
- **Audit Logging**: Log provider used, raw query, and final outcome classification (1-4).
- **Auto-Approval Lock**: Auto-approval only happens in Outcome 2. Outcome 4 MUST trigger manual review.
- **Verification**: The final implementation must be verified against `gemini-plan-eodhd.plan`.

## Technical Requirements
- **Async I/O**: Use `httpx` for external API calls.
- **Caching**: Local cache for external results (optional but recommended).
- **TDD**: 100% test coverage for resolution logic using mocks/fixtures.
