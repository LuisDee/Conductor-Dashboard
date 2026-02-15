# Rules Engine Refactor

**Goal:** Simplify and align the Rules Engine UI/logic with the latest product requirements, focusing on standardizing severity options, consolidating risk factors, and improving the audit trail.

## 1. Risk Factors — Instrument Type
- [x] **Simplify Config:** Remove granular instrument type breakdown.
- [x] **Add Toggles:** Implement `is_derivative` and `is_leveraged` toggles with severity options (Strongly Advise Rejection | High | Medium | Low).
- [x] **Add Info Panel:** Display read-only classification reference for Derivatives, Non-derivatives, and "Requires Clarification".
- [x] **Fix UI:** Ensure selected icon text color has sufficient contrast.

## 2. Risk Factors — Direction Match
- [x] **Invert Logic:** Replace single "high trigger" with two subsections: "Opposite to Mako position" and "Same as Mako position".
- [x] **Add Severity:** Add standard severity options to both new subsections.

## 3. Risk Factors — Employee Role
- [x] **Replace Component:** Reuse the tiered employee role component from Settings page (High/Medium tiers, others Low).

## 4. Risk Factors — Position Size
- [x] **Replace Component:** Reuse the position size slider component from Settings page.

## 5. Risk Factors — Connected Person
- [x] **Simplify:** Replace granular options with a single "Connected person detected" toggle and standard severity options.

## 6. Risk Factors — Restricted List
- [x] **Remove:** Delete this entry from Risk Factors (handled in Conflicts).

## 7. Risk Factors — Resolution
- [x] **Rename:** Change "Resolution" to "Unknown Instrument Resolution".
- [x] **Update Severity:** Set options to standard set.

## 8. Risk Factors — Price Discovery
- [x] **Update Severity:** Ensure options match the standard set.

## 9. Conflicts — Front Running
- [x] **Rename:** Change to "Mako Trading Lookback".
- [x] **Align UI:** Use "Mako Trading Lookback (Days)" component from Settings.
- [x] **Update Severity:** Set options to standard set.

## 10. Conflicts — Clean Up
- [x] **Remove:** Cross Trading (covered by Risk Factors).
- [x] **Remove:** Prohibited Instruments (covered by Risk Factors).

## 11. Conflicts — Restricted Instruments List
- [x] **Simplify:** Remove "Action on match" and "Block trade" sections.
- [x] **Update Severity:** Keep only the advisory signal with standard severity options.

## 12. Breaches — Severity Updates
- [x] **Update Severity:** Apply standard severity options to Execution Overdue, Contract Note, Approval Expired, and Holding Violation.

## 13. Advisory Section
- [x] **Remove Options:** Remove "Strongly Advise Rejection" and "Proceed with Caution".
- [x] **Add Toggle:** Add "Auto-approve low risk trades" (On/Off).

## 14. Audit Log & Save
- [x] **Enhance Log:** Ensure structured logging of all changes (who, what, when).
- [x] **Add Global Save:** Implement "Save All Changes" button that applies pending changes and writes a single audit log entry.
