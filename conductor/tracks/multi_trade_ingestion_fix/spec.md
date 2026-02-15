# Track Specification: Multi-Trade Ingestion Fix

## Problem Statement
The current email ingestion and manual upload pipeline only processes the first trade found in a document, even when the document is an activity statement containing multiple trades. Additionally, the dev environment is incorrectly defaulting to mock Slack even when real Slack mode is desired.

## Objectives
1.  **Enable Multi-Trade Extraction**: Ensure that all trades within an activity statement are extracted and processed.
2.  **Independent Routing**: Each extracted trade must be independently matched to users and routed (auto-approve, manual review, etc.).
3.  **Slack Configuration Alignment**: Fix the environment settings so the poller uses the real Slack API.

## Success Criteria
- [ ] An activity statement with 2+ trades is emailed to the dev mailbox.
- [ ] All trades are extracted and correctly matched to their respective PAD requests.
- [ ] PAD Execution records are created for all matching trades.
- [ ] Notifications (if any) are sent to the real Slack compliance channel.

## Technical Details
- **Core Service**: `src/pa_dealing/services/trade_document_processor.py`
- **Agent**: `src/pa_dealing/agents/document_processor/agent.py`
- **Configuration**: `.env` (linked to `.env.dev`)
