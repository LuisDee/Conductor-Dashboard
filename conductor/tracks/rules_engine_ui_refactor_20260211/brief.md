# Track Brief: Rules Engine UI Refactor

**Goal**: Refactor and improve the Rules Engine page to simplify risk factors, standardize severity options, and enhance audit logging.

**Source**: User request based on `@plan-rules-engine`.

## Context
The current Rules Engine page has granular breakdowns for instrument types and other factors that are too complex. The severity options are inconsistent. We need to simplify the UI to align with the new risk scoring model and ensure a consistent user experience.

## Requirements
- **Instrument Type**: Simplify to "Is Derivative" and "Is Leveraged" only.
- **Direction Match**: Split into "Opposite to Mako" and "Same as Mako".
- **Employee Role / Position Size**: Replace with Settings page components.
- **Connected Person**: Simplify to single field.
- **Restricted List**: Remove from Risk Factors.
- **Severity**: Standardize to `Strongly Advise Rejection` / `High` / `Medium` / `Low` everywhere.
- **Advisory**: Remove "Strongly Advise Rejection" and "Proceed with Caution" top-level options; add Auto-Approve toggle.
- **Audit**: Add "Save" button and log all changes.

## Deliverables
- Updated `RulesEngine.tsx` and child components.
- Standardized severity selectors.
- Audit logging for rule updates.
