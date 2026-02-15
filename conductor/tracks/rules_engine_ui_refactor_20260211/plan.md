# Rules Engine — Actionable Checklist

## Risk Factors Section

- [x] **Instrument Type — Simplify to two risk signals only**
  - [x] Remove the per-instrument-type granular breakdown from the UI
  - [x] Replace with only two configurable risk fields: **Is Derivative** and **Is Leveraged**
  - [x] Each field gets severity options: `Strongly Advise Rejection` / `High` / `Medium` / `Low`
  - [x] Display the derivative classification reference (O, F, f, V, w, s, r = derivative; S, c, D, b = non-derivative; C, x, I, B, R = prompt user for clarity) as an informational/read-only section so users understand the mapping
  - [x] **Save bar**: Comment field + Save Changes button in header (greyed out until changes detected)

- [x] **Mako Traded** — No changes needed

- [x] **Direction Match — Invert logic & split into two fields**
  - [x] Remove the current single "high trigger" control
  - [x] Add field: **Opposite to Mako Position** with severity options: `Strongly Advise Rejection` / `High` / `Medium` / `Low`
  - [x] Add field: **Same as Mako Position** with severity options: `Strongly Advise Rejection` / `High` / `Medium` / `Low`

- [x] **Employee Role — Replace with settings-page component**
  - [x] Remove current employee role UI
  - [x] Replace with High/Medium department tiers (matching Settings page pattern, everything else defaults to Low)

- [x] **Position Size — Replace with settings-page slider**
  - [x] Remove current basic position size UI
  - [x] Use the navy DualRangeSlider component from Settings page (logarithmic, dark navy gradient)
  - [x] Range extended to £10M

- [x] **Connected Person — Simplify**
  - [x] Remove granular connection-type breakdown (spouse, partner, child, etc.)
  - [x] Replace with a single field: **Connected Person Detected** with severity options: `Strongly Advise Rejection` / `High` / `Medium` / `Low`

- [x] **Holding Period** — No changes needed

- [x] **Restricted List — Remove**
  - [x] Remove entirely from the Risk Factors section (handled in Conflicts as CFL-004)

- [x] **Resolution — Rename & standardise**
  - [x] Rename section to **Unknown Instrument Resolution**
  - [x] Set severity options to: `Strongly Advise Rejection` / `High` / `Medium` / `Low`

- [x] **Price Discovery — Standardise options**
  - [x] Ensure severity options are: `Strongly Advise Rejection` / `High` / `Medium` / `Low`

---

## Conflicts Section

- [x] **Front Running — Rename & simplify**
  - [x] Rename to **Mako Trading Lookback**
  - [x] Set severity options to: `Strongly Advise Rejection` / `High` / `Medium` / `Low`

- [x] **Cross Trading — Remove**
  - [x] Remove entirely (already covered by Risk Factors)

- [x] **Prohibited Instruments — Remove**
  - [x] Remove entirely (already covered by Risk Factors)

- [x] **Restricted Instruments List — Simplify**
  - [x] Remove the "Action on Match" dropdown
  - [x] Remove the "Block trade / What happens when a security is on the Restricted List" section
  - [x] Keep only a single severity selector: `Strongly Advise Rejection` / `High` / `Medium` / `Low`

---

## Breaches Section

- [x] **Execution Overdue** — Standardise severity to: `Strongly Advise Rejection` / `High` / `Medium` / `Low`
- [x] **Contract Note** — Standardise severity to: `Strongly Advise Rejection` / `High` / `Medium` / `Low`
- [x] **Approval Expired** — Standardise severity to: `Strongly Advise Rejection` / `High` / `Medium` / `Low`
- [x] **Holding Violation** — Standardise severity to: `Strongly Advise Rejection` / `High` / `Medium` / `Low`

---

## Advisory Section

- [x] Remove the **Strongly Advise Rejection** advisory (ADV-001) — now individually configurable per rule
- [x] Remove the **Proceed with Caution** advisory (ADV-002)
- [x] Add a toggle: **Auto-Approve Low Risk Trades** (`On` / `Off`)

---

## Settings Migration (NEW — deprecate Settings page)

- [x] **Migrate all Settings page rules to the Rules Engine**
  - [x] Default Currency → `SET-001` rule
  - [x] Escalation Intervals (chasing manager/compliance/SMF16 hours) → `SET-002` rule
  - [x] Position Size Thresholds → already covered by `RF-005` (now with navy DualRangeSlider)
  - [x] Department Risk Categories → already covered by `RF-004` (High/Medium tiers)
  - [x] Advisory Criteria → replaced by per-rule severity selectors + `ADV-001` auto-approve toggle
  - [x] Holding Period, Execution Deadline, Contract Note Deadline → already `RF-007`, `BR-001`, `BR-002`
  - [x] Mako Lookback Days → already `RF-002` and `CFL-001`

---

## Dashboard Fixes (NEW)

- [x] **Recent Activity / Request Statistics height alignment**
  - [x] Both panels now stretch to equal height so their bottoms align horizontally

- [x] **Sidebar notification badges**
  - [x] Add notification count badge for Execution Tracking (cyan, matching dashboard tile)
  - [x] Add notification count badge for Mako Conflicts (purple, matching dashboard tile)
  - [x] Matches existing pattern used by Breaches and Pending Approvals

---

## Audit Log

- [x] Display audit log entries for all updates to the rules engine (Audit Log tab with `RuleAuditLog` component)
- [x] Ensure structured logging is emitted for every change (backend `routes/rules.py` logs to `pad_rule_audit` + central `audit_log`)
- [x] Add a **Save Changes** button with comment field in the page header (fades in when changes detected)
- [x] On save, insert an audit log record capturing: who made the change, what changed, and timestamp

---

## Settings Deprecation

- [x] Remove Settings route from `App.tsx`
- [x] Remove Settings from sidebar navigation
- [x] All settings accessible via Rules Engine > Settings tab

---

## Database Migration

- [x] Add Alembic migration `20260212_1000_refactor_pad_rule_schemas.py`
  - [x] Remove old rules: RF-008, CFL-002, CFL-003, old ADV-001, ADV-002
  - [x] Add new rules: ADV-001 (auto-approve toggle), SET-001 (currency), SET-002 (escalation)
  - [x] Update existing rules with new config structures and renames

---

## Global / Cross-cutting

- [x] Ensure **all** severity selectors across the entire rules engine use the consistent set: `Strongly Advise Rejection` / `High` / `Medium` / `Low`
- [x] Save bar with comment field in header (replaces old sticky bottom bar; original "icon text colour bug" was in lost code)
