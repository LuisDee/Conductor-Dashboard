# Track: LiteLLM Proxy Migration

## Objective
Migrate the project's agents from direct Vertex AI usage to the LiteLLM proxy hosted at `https://litellm.production.mako-cloud.com`.

## Goals
- [x] Research ADK support for LiteLLM custom base URLs.
- [x] Update `Settings` to include LiteLLM API key and Base URL.
- [x] Configure `Agent` instances to use the LiteLLM proxy.
- [x] Verify connectivity and performance (consider `gemini-3-flash`).
- [x] Update Docker environments with new environment variables.

## Plan
1. **Research:** Read ADK documentation regarding custom models via LiteLLM. (COMPLETED)
2. **Configuration:** Add `LITELLM_API_KEY` and `LITELLM_BASE_URL` to `settings.py`. (COMPLETED)
3. **Implementation:** Update `src/pa_dealing/agents/slack/chatbot.py` and other agent factories. (COMPLETED)
4. **Verification:** Run integration tests to ensure the proxy is routing correctly. (COMPLETED)
