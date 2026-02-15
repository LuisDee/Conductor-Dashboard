# Spec: PAD Search Row Dimming Bug

## Problem Statement
Filter sorting works (matching rows move to the top), but non-matching rows are not visually dimmed as expected.

### Current State
With active filters (e.g., SYMBOL:AAPL, DESK:US_D1):
- Matching rows appear at the top ✓
- Non-matching rows appear below but are **not** visually dimmed ✗

### Expected Behavior
Non-matching rows should have reduced opacity (e.g., `opacity: 0.5`) or a specific "dimmed" CSS class applied to distinguish them from active matches.

## Investigation Points
- [x] **Data Logic**: How is `dimmedMakoIds` (or equivalent) being computed? (Switched to Set<string> with composite keys).
- [x] **Prop Propagation**: Is `rowClassName` receiving the dimmed Set and applying the correct class/style? (Yes, verified).
- [x] **CSS Overrides**: Is the `Table` component's inline background/hover styles overriding the opacity? (Fixed by disabling hover bg on dimmed rows).
- [x] **CSS Existence**: Verify the CSS class for dimming exists and has correct styles/specificity. (Added .row-dimmed with !important).
- [x] **Execution**: Console.log inside `rowClassName` to verify it is being called and returning the expected values for non-matching rows. (Added logs).
- [x] **Identity Matching**: Check for Row ID mismatches between the dimmed Set and the row object. (Fixed with composite keys).

## Likely Causes
1. Dimmed Set is empty or stale (mutable variable anti-pattern).
2. `rowClassName` returns the class but CSS doesn't apply (specificity or missing styles).
3. Row ID mismatch between what's in the Set vs what's on the row object.
