# Specification: Rules Engine UI Refactor

## Overview
Refactor the Rules Engine UI to simplify configuration, enforce consistent severity levels, and improve auditability.

## Requirements

### Risk Factors Section
- **Instrument Type**
  - Remove per-instrument-type breakdown.
  - Add configurable fields: **Is Derivative**, **Is Leveraged**.
  - Show derivative classification reference as read-only info.
  - Fix selected icon text color bug.
- **Direction Match**
  - Remove single "high trigger" control.
  - Add fields: **Opposite to Mako Position**, **Same as Mako Position**.
- **Employee Role**
  - Replace current UI with Settings page component (High/Medium tiers + Add button).
- **Position Size**
  - Replace current UI with Settings page slider.
- **Connected Person**
  - Simplify to single field: **Connected Person Detected**.
- **Restricted List**
  - Remove from Risk Factors section.
- **Resolution**
  - Rename to **Unknown Instrument Resolution**.
- **Price Discovery**
  - Standardize severity options.

### Conflicts Section
- **Front Running**
  - Rename to **Mako Trading Lookback**.
  - Align with Settings page concept.
- **Cross Trading**
  - Remove entirely.
- **Prohibited Instruments**
  - Remove entirely.
- **Restricted Instruments List**
  - Remove "Action on Match" and "Block trade" sections.
  - Keep single severity selector.

### Breaches Section
- Standardize severity for **Execution Overdue**, **Contract Note**, **Approval Expired**, **Holding Violation**.

### Advisory Section
- Remove **Strongly Advise Rejection** and **Proceed with Caution** options.
- Add **Auto-Approve Low Risk Trades** toggle.

### Audit Log
- Add **Save All Changes** button.
- Log all changes to `audit_log` table.
- Display audit log entries.

### Global
- Ensure consistent severity options: `Strongly Advise Rejection` / `High` / `Medium` / `Low`.
- Fix selected-icon text color bug.
