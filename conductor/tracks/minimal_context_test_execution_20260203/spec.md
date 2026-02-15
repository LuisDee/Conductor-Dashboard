# Specification: Minimal-Context Test Execution System

## Problem
Test output bloats the agent context window, consuming tokens and degrading performance. We need tests to run with output redirected to log files, returning only pass/fail status and minimal failure context to the agent.

## Objectives
- Redirect all test output to `/tmp/agent_tests/`.
- Provide synchronous and background execution modes.
- Implement specialized failure extraction for Pytest and Playwright.
- Create a Gemini Skill to enforce usage of the new system.

## Extraction Logic
- **Pytest**: FAILED/ERROR lines, short test summary info, and limited assertion details.
- **Playwright**: Failed test names and first error/expect message.
- **Other**: First 20 lines of log output.
