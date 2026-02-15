# Implementation Plan: Multi-Trade Ingestion Fix

## Step 1: Fix Slack Configuration
- **Action**: Update `.env` (or `.env.dev`) to set `SLACK_MODE=real` and `SLACK_API_BASE_URL=https://slack.com/api/`.
- **Verification**: Run `docker compose up -d graph-email-poller` and check logs for the `SLACK_MODE is REAL` warning (it should no longer show the mock URL).

## Step 2: Refactor Document Agent usage
- **Action**: Modify `process_trade_document` in `src/pa_dealing/services/trade_document_processor.py` to call `doc_agent.extract_all_trades()` by default.
- **Goal**: Always receive a list of trades, even if the list contains only one item (for contract notes).

## Step 3: Update Extraction Routing
- **Action**: Ensure the loop in `process_trade_document` correctly handles the output of `ExtractionRouter` for every trade in the list.
- **Action**: Verify that `ParsedTrade` records are created for each unique trade found.

## Step 4: Verification
- **Test**: Use the `scripts/ops/run_graph_email_poller.py --once` command to process the user's latest email again (after clearing the idempotency state if necessary).
- **Verify**: Check the `pad_execution` and `parsed_trade` tables for multiple entries linked to the same document.
