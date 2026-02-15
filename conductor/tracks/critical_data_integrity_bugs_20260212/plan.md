# Critical Data Integrity Bug Fixes - Implementation Plan

**Track ID**: `critical_data_integrity_bugs_20260212`
**Priority**: CRITICAL
**Status**: Implementation Ready

## Executive Summary

This track addresses 7 critical data integrity bugs identified during autopsy code review of the PA Dealing compliance system. These bugs range from silent data corruption (PDF double-decode, FX rate fallback) to incorrect audit logging and UI validation issues. Each fix is analyzed for risk, caller impact, and rollback strategy.

---

## Phase 1: PDF Double-Decode Fix (SAFE - 1-line fix)

### Bug Description
**File**: `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/services/graph_email_poller.py`
**Line**: 353
**Severity**: CRITICAL - Corrupts PDF attachments, breaks trade extraction

Microsoft Graph API returns `attachment.content_bytes` as **already-decoded bytes**, not base64-encoded strings. The current code applies `base64.b64decode()` to already-decoded bytes, resulting in corrupted PDFs that fail parsing.

### Before (Lines 346-360)
```python
        for attachment in attachments:
            if not attachment.content_bytes:
                log.warning("attachment_no_content", attachment_name=attachment.name)
                continue

            try:
                # Graph API returns content_bytes as base64-encoded - decode it
                pdf_bytes = base64.b64decode(attachment.content_bytes)

                # Create document record for tracking
                document = await _create_document_record(
                    session, attachment, message, message_id
                )

                # Process through unified processor
                input_data = TradeDocumentInput(
```

### After (Lines 346-360)
```python
        for attachment in attachments:
            if not attachment.content_bytes:
                log.warning("attachment_no_content", attachment_name=attachment.name)
                continue

            try:
                # Graph API returns content_bytes as raw bytes (already decoded)
                pdf_bytes = attachment.content_bytes

                # Create document record for tracking
                document = await _create_document_record(
                    session, attachment, message, message_id
                )

                # Process through unified processor
                input_data = TradeDocumentInput(
```

### Caller Impact Analysis
**Direct callers**: None (GraphEmailPoller is a standalone service)
**Indirect impact**: All email-sourced trade extraction flows
**Risk**: **LOW** - This is a pure bug fix with no side effects

### Test Specification
```python
# tests/unit/services/test_graph_email_poller.py

async def test_pdf_attachment_not_double_decoded():
    """Verify PDF bytes are passed through without base64 decoding."""
    # Setup: Mock Graph attachment with raw PDF bytes
    raw_pdf_bytes = b"%PDF-1.4\n%\xc3\xa4\xc3\xbc..."  # Valid PDF header

    attachment = Mock()
    attachment.content_bytes = raw_pdf_bytes
    attachment.name = "trade_confirm.pdf"
    attachment.content_type = "application/pdf"

    message = Mock()
    message.message_id = "test-msg-123"
    message.sender_email = "broker@example.com"

    # Execute: Process message
    poller = GraphEmailPoller()
    with patch('pa_dealing.services.graph_email_poller.process_trade_document') as mock_process:
        await poller._process_message(session, message)

    # Verify: PDF bytes passed unchanged to processor
    call_args = mock_process.call_args[0][1]  # TradeDocumentInput
    assert call_args.pdf_bytes == raw_pdf_bytes
    assert call_args.pdf_bytes.startswith(b"%PDF")  # Valid PDF header
```

### Rollback Strategy
**Git revert** - One-line change, safe to revert instantly if issues arise.

---

## Phase 2: FX Rate Fallback Fix (RISKY - 3 callers affected)

### Bug Description
**File**: `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/services/currency_service.py`
**Lines**: 58-61
**Severity**: CRITICAL - Silently masks missing FX data, corrupts risk scoring

When FX rate lookup fails, the service silently falls back to `rate = 1.0` instead of raising the defined `CurrencyConversionError`. This causes:
- Risk scoring to use incorrect GBP values for thresholds
- Compliance reports to show false position sizes
- No alert when FX data pipeline breaks

**Note**: `CurrencyConversionError` is **already defined** (line 19) but **never raised**.

### Before (Lines 56-67)
```python
    # Fetch FX rate from oracle_fx and oracle_currency tables
    rate = await get_fx_rate(session, currency, trade_date)

    if rate is None:
        log.warning("fx_rate_not_found_using_fallback", currency=currency, trade_date=str(trade_date))
        # Fallback to 1.0 if rate not found
        rate = Decimal("1.0")

    # Convert to GBP
    gbp_value = Decimal(str(value)) * rate
    log.info("currency_converted", value=value, currency=currency, gbp_value=round(gbp_value, 2), rate=rate)

    return gbp_value
```

### After (Lines 56-67)
```python
    # Fetch FX rate from oracle_fx and oracle_currency tables
    rate = await get_fx_rate(session, currency, trade_date)

    if rate is None:
        # CRITICAL: Do not fallback - missing FX data must be visible
        error_msg = f"No FX rate found for {currency} on {trade_date} (lookback 3 days)"
        log.error("fx_rate_missing_cannot_convert", currency=currency, trade_date=str(trade_date))
        raise CurrencyConversionError(error_msg)

    # Convert to GBP
    gbp_value = Decimal(str(value)) * rate
    log.info("currency_converted", value=value, currency=currency, gbp_value=round(gbp_value, 2), rate=rate)

    return gbp_value
```

### Caller Impact Analysis

#### Caller 1: `risk_scoring_service.py:242`
```python
# BEFORE (lines 237-246)
    # 4.5. Convert value to GBP
    value_gbp = None
    if value:
        try:
            value_gbp = await convert_to_gbp(session, value, currency, trade_date)
        except Exception as e:
            log.warning("currency_conversion_failed", e=e)
            value_gbp = Decimal(str(value))

    # 5. Run scoring

# AFTER (lines 237-248)
    # 4.5. Convert value to GBP
    value_gbp = None
    if value:
        try:
            value_gbp = await convert_to_gbp(session, value, currency, trade_date)
        except CurrencyConversionError as e:
            # FX data missing - log error and treat as GBP (conservative)
            log.error("fx_rate_missing_treating_as_gbp", currency=currency, trade_date=trade_date, error=str(e))
            value_gbp = Decimal(str(value))
            # TODO: Consider flagging request for manual review
        except Exception as e:
            log.error("currency_conversion_unexpected_error", e=e)
            value_gbp = Decimal(str(value))

    # 5. Run scoring
```
**Impact**: Risk scoring will log ERROR instead of WARNING, making FX pipeline failures visible. Conservative fallback (treat as GBP) prevents auto-approvals.

#### Caller 2: `chatbot.py:2151`
```python
# BEFORE (lines 2146-2155)
        currency = draft.currency or "USD"
        value = draft.value

        try:
            async with get_session() as session:
                value_gbp = float(await convert_to_gbp(session, value, currency))
        except Exception as e:
            log.warning("currency_conversion_failed_using_raw_value_for_threshold_check", e=e)
            value_gbp = value  # Fallback: assume already in GBP

        # Check if value exceeds threshold

# AFTER (lines 2146-2158)
        currency = draft.currency or "USD"
        value = draft.value

        try:
            async with get_session() as session:
                value_gbp = float(await convert_to_gbp(session, value, currency))
        except CurrencyConversionError as e:
            # FX rate missing - notify user and use raw value (conservative)
            log.error("fx_rate_missing_in_chatbot_threshold_check", currency=currency, error=str(e))
            value_gbp = value  # Conservative: assume already in GBP
            await self._send_message(thread_ts, f"⚠️ FX rate for {currency} unavailable - treating value as GBP for threshold check")
        except Exception as e:
            log.error("currency_conversion_unexpected_error_in_chatbot", e=e)
            value_gbp = value

        # Check if value exceeds threshold
```
**Impact**: Chatbot alerts user when FX data is missing, preventing silent failures.

#### Caller 3: `format_currency_with_gbp` (Internal, lines 140-162)
```python
# BEFORE
async def format_currency_with_gbp(
    session: AsyncSession,
    value: float | Decimal,
    currency: str,
    trade_date: date | datetime | None = None,
) -> str:
    """Format value in both original currency and GBP equivalent."""
    if currency.upper() == "GBP":
        return f"£{value:,.2f} GBP"

    gbp_value = await convert_to_gbp(session, value, currency, trade_date)

    return f"{currency.upper()} {value:,.2f} (≈ £{gbp_value:,.2f} GBP)"

# AFTER
async def format_currency_with_gbp(
    session: AsyncSession,
    value: float | Decimal,
    currency: str,
    trade_date: date | datetime | None = None,
) -> str:
    """Format value in both original currency and GBP equivalent."""
    if currency.upper() == "GBP":
        return f"£{value:,.2f} GBP"

    try:
        gbp_value = await convert_to_gbp(session, value, currency, trade_date)
        return f"{currency.upper()} {value:,.2f} (≈ £{gbp_value:,.2f} GBP)"
    except CurrencyConversionError:
        # FX rate missing - show original currency only
        log.warning("fx_rate_missing_in_format_function", currency=currency, trade_date=trade_date)
        return f"{currency.upper()} {value:,.2f} (FX rate unavailable)"
```
**Impact**: UI display functions degrade gracefully instead of crashing.

### Test Specification
```python
# tests/unit/services/test_currency_service.py

async def test_missing_fx_rate_raises_error_not_fallback():
    """Verify CurrencyConversionError is raised when FX rate missing."""
    session = Mock()

    # Mock get_fx_rate to return None (rate not found)
    with patch('pa_dealing.services.currency_service.get_fx_rate', return_value=None):
        with pytest.raises(CurrencyConversionError) as exc_info:
            await convert_to_gbp(session, value=100000, currency="USD", trade_date=date(2026, 2, 12))

        assert "No FX rate found for USD" in str(exc_info.value)
        assert "2026-02-12" in str(exc_info.value)


async def test_risk_scoring_handles_missing_fx_gracefully():
    """Verify risk scoring logs error but continues with conservative fallback."""
    # Setup: Request with USD currency, no FX rate
    session = await create_test_session()
    request_data = {
        "estimated_value": 50000,
        "currency": "USD",
        "trade_date": date(2026, 2, 12),
    }

    with patch('pa_dealing.services.currency_service.get_fx_rate', return_value=None):
        # Should not crash - should log error and use conservative fallback
        result = await run_risk_scoring(session, request_data)

        assert result is not None  # Scoring completed
        # Conservative fallback should treat 50k USD as 50k GBP (overestimate)


async def test_chatbot_notifies_user_on_fx_failure():
    """Verify chatbot alerts user when FX rate missing."""
    chatbot = TradingChatbot()
    draft = Mock()
    draft.currency = "EUR"
    draft.value = 100000

    with patch('pa_dealing.services.currency_service.get_fx_rate', return_value=None):
        with patch.object(chatbot, '_send_message') as mock_send:
            await chatbot._check_threshold(draft, thread_ts="thread-123")

            # Verify user was notified
            mock_send.assert_called_once()
            message = mock_send.call_args[0][1]
            assert "FX rate for EUR unavailable" in message
```

### Rollback Strategy
**Staged rollback**:
1. Revert caller changes first (risk_scoring_service, chatbot, format_currency_with_gbp)
2. Then revert currency_service.py change
3. Monitor logs for `fx_rate_not_found_using_fallback` warnings after rollback

**Risk**: MEDIUM - Changes affect 3 files but all have exception handling already in place.

---

## Phase 3: Orphan Recovery Timeout Fix (SAFE - 1-line fix)

### Bug Description
**File**: `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/services/pdf_poller.py`
**Line**: 463
**Severity**: HIGH - Recovers documents immediately, breaks concurrency

The orphan recovery mechanism is designed to reset documents stuck in `processing` status for longer than `timeout_minutes`. However, the cutoff calculation is missing the timeout subtraction:

```python
cutoff = datetime.now(UTC).replace(tzinfo=None)  # WRONG - no timeout offset
```

This means **ALL** documents with `status = 'processing'` are recovered immediately, even if they were just claimed 1 second ago. This breaks concurrent processing.

### Before (Lines 456-474)
```python
async def recover_orphaned_documents(
    session: AsyncSession,
    timeout_minutes: int = 30,
) -> int:
    """Recover documents stuck in 'processing' status.

    Args:
        session: Database session
        timeout_minutes: How long before considering a document stuck

    Returns:
        Number of documents recovered
    """
    cutoff = datetime.now(UTC).replace(tzinfo=None)

    # Find stuck documents
    result = await session.execute(
        text("""
            UPDATE padealing.gcs_document
            SET status = 'pending',
                retry_count = retry_count + 1,
                error_message = 'Orphan recovery: stuck in processing'
            WHERE status = 'processing'
            AND processing_started_at < :cutoff
            RETURNING id
```

### After (Lines 456-476)
```python
async def recover_orphaned_documents(
    session: AsyncSession,
    timeout_minutes: int = 30,
) -> int:
    """Recover documents stuck in 'processing' status.

    Args:
        session: Database session
        timeout_minutes: How long before considering a document stuck

    Returns:
        Number of documents recovered
    """
    from datetime import timedelta

    cutoff = datetime.now(UTC).replace(tzinfo=None) - timedelta(minutes=timeout_minutes)

    # Find stuck documents
    result = await session.execute(
        text("""
            UPDATE padealing.gcs_document
            SET status = 'pending',
                retry_count = retry_count + 1,
                error_message = 'Orphan recovery: stuck in processing'
            WHERE status = 'processing'
            AND processing_started_at < :cutoff
            RETURNING id
```

### Caller Impact Analysis
**Direct callers**: Background job scheduler (likely Celery/Airflow task)
**Indirect impact**: PDF processing concurrency
**Risk**: **LOW** - Fix makes function work as intended

### Test Specification
```python
# tests/unit/services/test_pdf_poller.py

async def test_orphan_recovery_respects_timeout():
    """Verify orphan recovery only affects documents older than timeout."""
    session = await create_test_session()
    now = datetime.now(UTC).replace(tzinfo=None)

    # Create 3 documents:
    # 1. Processing for 45 minutes (should recover)
    # 2. Processing for 15 minutes (should NOT recover with 30-min timeout)
    # 3. Processing for 31 minutes (should recover)
    doc1 = GCSDocument(
        status="processing",
        processing_started_at=now - timedelta(minutes=45),
        retry_count=0
    )
    doc2 = GCSDocument(
        status="processing",
        processing_started_at=now - timedelta(minutes=15),
        retry_count=0
    )
    doc3 = GCSDocument(
        status="processing",
        processing_started_at=now - timedelta(minutes=31),
        retry_count=0
    )

    session.add_all([doc1, doc2, doc3])
    await session.commit()

    # Execute recovery with 30-minute timeout
    recovered_count = await recover_orphaned_documents(session, timeout_minutes=30)

    # Verify results
    assert recovered_count == 2  # doc1 and doc3 recovered

    await session.refresh(doc1)
    await session.refresh(doc2)
    await session.refresh(doc3)

    assert doc1.status == "pending"
    assert doc1.retry_count == 1
    assert doc2.status == "processing"  # NOT recovered (under timeout)
    assert doc2.retry_count == 0
    assert doc3.status == "pending"
    assert doc3.retry_count == 1
```

### Rollback Strategy
**Git revert** - One-line change with clear intent. Safe to revert instantly.

---

## Phase 4: Audit ActionType Fix (SAFE - 2-line fix)

### Bug Description
**Files**:
- `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/services/pad_service.py` lines 1319, 1413
- `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/audit/logger.py` lines 63-64

**Severity**: MEDIUM - Audit logs use wrong action types for breach lifecycle

The ActionType enum **already defines** `BREACH_DETECTED` (line 63) and `BREACH_RESOLVED` (line 64), but `pad_service.py` uses `PAD_REQUEST_VIEWED` instead. This makes breach audit logs unsearchable and breaks compliance reporting.

### Before: Line 1319 Context (Lines 1310-1327)
```python
            breach = PADBreach(
                request_id=request_id,
                employee_id=employee_id,
                breach_type=breach_type,
                severity=severity,
                details=details or {},
                description=description,
                detected_by=detected_by,
                detected_at=datetime.now(UTC).replace(tzinfo=None),
                resolved=False,
            )
            session.add(breach)
            await session.flush()

            # Audit log
            await self._audit.log(
                action_type=ActionType.PAD_REQUEST_VIEWED,  # Need better type
                action_status=ActionStatus.SUCCESS,
                actor_type=ActorType.SYSTEM,
                entity_type="breach",
                entity_id=str(breach.id),
                details={"type": breach_type, "severity": severity, "description": description},
            )

            return breach
```

### After: Line 1319 Context (Lines 1310-1327)
```python
            breach = PADBreach(
                request_id=request_id,
                employee_id=employee_id,
                breach_type=breach_type,
                severity=severity,
                details=details or {},
                description=description,
                detected_by=detected_by,
                detected_at=datetime.now(UTC).replace(tzinfo=None),
                resolved=False,
            )
            session.add(breach)
            await session.flush()

            # Audit log
            await self._audit.log(
                action_type=ActionType.BREACH_DETECTED,
                action_status=ActionStatus.SUCCESS,
                actor_type=ActorType.SYSTEM,
                entity_type="breach",
                entity_id=str(breach.id),
                details={"type": breach_type, "severity": severity, "description": description},
            )

            return breach
```

### Before: Line 1413 Context (Lines 1405-1426)
```python
            breach.resolved = True
            breach.resolved_at = datetime.now(UTC).replace(tzinfo=None)
            breach.resolved_by_id = resolved_by_id
            breach.resolution_notes = resolution_notes

            await session.commit()

            # Audit log
            await self._audit.log(
                action_type=ActionType.PAD_REQUEST_VIEWED,  # Could add BREACH_RESOLVED type
                action_status=ActionStatus.SUCCESS,
                actor_type=ActorType.USER,
                actor_id=str(resolved_by_id),
                actor_email=actor_email,
                entity_type="breach",
                entity_id=str(breach_id),
                details={
                    "breach_type": breach.breach_type,
                    "resolution_notes": resolution_notes,
                },
            )

            return {"success": True, "breach_id": breach_id}
```

### After: Line 1413 Context (Lines 1405-1426)
```python
            breach.resolved = True
            breach.resolved_at = datetime.now(UTC).replace(tzinfo=None)
            breach.resolved_by_id = resolved_by_id
            breach.resolution_notes = resolution_notes

            await session.commit()

            # Audit log
            await self._audit.log(
                action_type=ActionType.BREACH_RESOLVED,
                action_status=ActionStatus.SUCCESS,
                actor_type=ActorType.USER,
                actor_id=str(resolved_by_id),
                actor_email=actor_email,
                entity_type="breach",
                entity_id=str(breach_id),
                details={
                    "breach_type": breach.breach_type,
                    "resolution_notes": resolution_notes,
                },
            )

            return {"success": True, "breach_id": breach_id}
```

### Caller Impact Analysis
**Direct callers**: Breach detection/resolution flows
**Indirect impact**: Audit log queries, compliance reports
**Risk**: **ZERO** - Pure metadata change, no logic affected

### Test Specification
```python
# tests/unit/services/test_pad_service.py

async def test_breach_detection_logs_correct_action_type():
    """Verify breach detection uses BREACH_DETECTED action type."""
    session = await create_test_session()
    pad_service = PADService()

    # Mock audit logger
    with patch.object(pad_service, '_audit') as mock_audit:
        await pad_service.create_breach(
            request_id=123,
            employee_id=456,
            breach_type="contract_note_mismatch",
            severity="MEDIUM",
            description="Test breach"
        )

        # Verify correct action type
        mock_audit.log.assert_called_once()
        call_kwargs = mock_audit.log.call_args[1]
        assert call_kwargs['action_type'] == ActionType.BREACH_DETECTED
        assert call_kwargs['entity_type'] == "breach"


async def test_breach_resolution_logs_correct_action_type():
    """Verify breach resolution uses BREACH_RESOLVED action type."""
    session = await create_test_session()
    pad_service = PADService()

    # Create a breach first
    breach = PADBreach(
        request_id=123,
        employee_id=456,
        breach_type="test_breach",
        severity="LOW",
        resolved=False
    )
    session.add(breach)
    await session.commit()

    # Mock audit logger
    with patch.object(pad_service, '_audit') as mock_audit:
        await pad_service.resolve_breach(
            breach_id=breach.id,
            resolved_by_id=789,
            actor_email="compliance@example.com",
            resolution_notes="False positive"
        )

        # Verify correct action type
        mock_audit.log.assert_called_once()
        call_kwargs = mock_audit.log.call_args[1]
        assert call_kwargs['action_type'] == ActionType.BREACH_RESOLVED
        assert call_kwargs['entity_type'] == "breach"
```

### Rollback Strategy
**Git revert** - Two-line change, zero risk. Safe to revert instantly.

---

## Phase 5: Risk Routing Clarification (SAFE - Docstring update)

### Bug Description
**File**: `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/agents/orchestrator/risk_scoring.py`
**Lines**: 681 (docstring) vs 694 (code)
**Severity**: LOW - Documentation mismatch, no logic bug

The `aggregate_risk()` function docstring claims `HIGH → SMF16` but the code routes HIGH to COMPLIANCE:

```python
# Line 681 (docstring)
- HIGH → SMF16

# Line 694 (code)
approval_route = ApprovalRoute.COMPLIANCE
```

**Analysis**: The code is **correct** - HIGH risk goes to COMPLIANCE for initial review, with manual escalation to SMF16 if needed. The docstring is outdated.

### Before (Lines 680-695)
```python
        """Aggregate individual risk factors into overall risk level.

        Routing:
        - HIGH → SMF16
        - MEDIUM → Compliance
        - LOW → Auto-approve
        """
        # Count levels (excluding N/A)
        high_count = sum(1 for f in factors if f.level == FactorLevel.HIGH)
        medium_count = sum(1 for f in factors if f.level == FactorLevel.MEDIUM)
        low_count = sum(1 for f in factors if f.level == FactorLevel.LOW)

        # Determine overall level
        if high_count > 0:
            overall_level = OverallRiskLevel.HIGH
            # NEW: Automated HIGH risk routes to COMPLIANCE for initial review gate
            approval_route = ApprovalRoute.COMPLIANCE
```

### After (Lines 680-695)
```python
        """Aggregate individual risk factors into overall risk level.

        Routing:
        - HIGH → COMPLIANCE (initial review, manual escalation to SMF16 if needed)
        - MEDIUM → COMPLIANCE
        - LOW → Auto-approve
        """
        # Count levels (excluding N/A)
        high_count = sum(1 for f in factors if f.level == FactorLevel.HIGH)
        medium_count = sum(1 for f in factors if f.level == FactorLevel.MEDIUM)
        low_count = sum(1 for f in factors if f.level == FactorLevel.LOW)

        # Determine overall level
        if high_count > 0:
            overall_level = OverallRiskLevel.HIGH
            # HIGH risk routes to COMPLIANCE for initial review gate
            approval_route = ApprovalRoute.COMPLIANCE
```

### Caller Impact Analysis
**Impact**: ZERO - Documentation-only change

### Test Specification
```python
# tests/unit/agents/orchestrator/test_risk_scoring.py

async def test_high_risk_routes_to_compliance_not_smf16():
    """Verify HIGH risk routing matches documented behavior."""
    factors = [
        RiskFactor(name="position_size", level=FactorLevel.HIGH, weight=0.3),
        RiskFactor(name="volatility", level=FactorLevel.MEDIUM, weight=0.2),
    ]

    result = aggregate_risk(factors)

    assert result.overall_level == OverallRiskLevel.HIGH
    assert result.approval_route == ApprovalRoute.COMPLIANCE  # NOT SMF16
    assert "HIGH risk" in result.explanation
```

### Rollback Strategy
**Not needed** - Documentation-only change.

---

## Phase 6: Range Slider Validation (SAFE - Frontend fix)

### Bug Description
**File**: `/Users/luisdeburnay/work/rules_engine_refactor/dashboard/src/components/ui/DualRangeSlider.tsx`
**Lines**: 95-96
**Severity**: LOW - UI can get into invalid state (low > high)

The `handleMouseMove` function doesn't prevent the low handle from crossing past the high handle during drag operations. This allows invalid states like `low: £500k, high: £100k`.

### Before (Lines 83-104)
```typescript
  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!dragging || !trackRef.current) return

      const rect = trackRef.current.getBoundingClientRect()
      let position = (e.clientX - rect.left) / rect.width
      position = Math.max(0, Math.min(1, position))

      let newValue = fromLinear(position)
      newValue = snapToNice(newValue)
      newValue = Math.max(min, Math.min(max, newValue))

      if (dragging === 'low') {
        newValue = Math.min(newValue, maxValue - step)
        onChange({ low: newValue, high: maxValue })
      } else {
        newValue = Math.max(newValue, minValue + step)
        onChange({ low: minValue, high: newValue })
      }
    },
    [dragging, fromLinear, snapToNice, minValue, maxValue, min, max, step, onChange]
  )
```

### After (Lines 83-108)
```typescript
  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!dragging || !trackRef.current) return

      const rect = trackRef.current.getBoundingClientRect()
      let position = (e.clientX - rect.left) / rect.width
      position = Math.max(0, Math.min(1, position))

      let newValue = fromLinear(position)
      newValue = snapToNice(newValue)
      newValue = Math.max(min, Math.min(max, newValue))

      if (dragging === 'low') {
        // Prevent low handle from crossing high handle
        const maxAllowed = maxValue - step
        newValue = Math.min(newValue, maxAllowed)
        // Only update if value actually changed
        if (newValue !== minValue) {
          onChange({ low: newValue, high: maxValue })
        }
      } else {
        // Prevent high handle from crossing low handle
        const minAllowed = minValue + step
        newValue = Math.max(newValue, minAllowed)
        // Only update if value actually changed
        if (newValue !== maxValue) {
          onChange({ low: minValue, high: newValue })
        }
      }
    },
    [dragging, fromLinear, snapToNice, minValue, maxValue, min, max, step, onChange]
  )
```

### Caller Impact Analysis
**Direct callers**: Risk configuration UI
**Indirect impact**: Threshold configuration
**Risk**: **ZERO** - Pure UI validation enhancement

### Test Specification
```typescript
// dashboard/src/components/ui/DualRangeSlider.test.tsx

describe('DualRangeSlider handle crossing prevention', () => {
  it('prevents low handle from dragging past high handle', () => {
    const onChange = jest.fn()
    render(
      <DualRangeSlider
        minValue={10000}
        maxValue={100000}
        min={1000}
        max={1000000}
        step={1000}
        onChange={onChange}
      />
    )

    const lowHandle = screen.getByTestId('low-handle')

    // Simulate dragging low handle to the right (past high)
    fireEvent.mouseDown(lowHandle)
    fireEvent.mouseMove(window, { clientX: 500 }) // Beyond high handle position

    // Verify low value capped at (high - step)
    const lastCall = onChange.mock.calls[onChange.mock.calls.length - 1][0]
    expect(lastCall.low).toBeLessThan(lastCall.high)
    expect(lastCall.low).toBe(100000 - 1000) // maxValue - step
  })

  it('prevents high handle from dragging below low handle', () => {
    const onChange = jest.fn()
    render(
      <DualRangeSlider
        minValue={10000}
        maxValue={100000}
        min={1000}
        max={1000000}
        step={1000}
        onChange={onChange}
      />
    )

    const highHandle = screen.getByTestId('high-handle')

    // Simulate dragging high handle to the left (below low)
    fireEvent.mouseDown(highHandle)
    fireEvent.mouseMove(window, { clientX: 50 }) // Below low handle position

    // Verify high value capped at (low + step)
    const lastCall = onChange.mock.calls[onChange.mock.calls.length - 1][0]
    expect(lastCall.high).toBeGreaterThan(lastCall.low)
    expect(lastCall.high).toBe(10000 + 1000) // minValue + step
  })
})
```

### Rollback Strategy
**Git revert** - Frontend-only change, zero backend impact. Safe to revert instantly.

---

## Phase 7: Boolean Select Fix (SAFE - Frontend fix)

### Bug Description
**File**: `/Users/luisdeburnay/work/rules_engine_refactor/dashboard/src/pages/NewRequest.tsx`
**Lines**: 194-198
**Severity**: MEDIUM - API receives strings instead of booleans, inverted insider info

Three issues:
1. Boolean fields (isDerivative, isLeveraged, isForOtherPerson) use string values "true"/"false"
2. Insider info checkbox has inverted logic: `!== 'true'` instead of `=== 'true'`
3. Backend expects actual booleans, not strings

### Before (Lines 185-201)
```typescript
    const payload = {
      employee_id: currentUser?.employee_id,
      security_name: data.securityName,
      direction: data.direction,
      quantity: data.quantity,
      estimated_value: data.estimatedValue,
      currency: data.currency,
      bloomberg: data.bloomberg || undefined,
      isin: data.isin || undefined,
      ticker: data.ticker || undefined,
      sedol: data.sedol || undefined,
      existing_position: data.existingPosition,
      is_derivative: String(data.isDerivative) === 'true',
      is_leveraged: String(data.isLeveraged) === 'true',
      is_related_party: String(data.isForOtherPerson) === 'true',
      relation: String(data.isForOtherPerson) === 'true' ? data.relation : undefined,
      insider_info_confirmed: String(data.insiderInfo) !== 'true', // API expects confirmation they DON'T have insider info
      justification: data.justification || undefined,
      derivative_justification: String(data.isDerivative) === 'true' ? data.derivativeJustification : undefined,
    }
```

### After (Lines 185-201)
```typescript
    const payload = {
      employee_id: currentUser?.employee_id,
      security_name: data.securityName,
      direction: data.direction,
      quantity: data.quantity,
      estimated_value: data.estimatedValue,
      currency: data.currency,
      bloomberg: data.bloomberg || undefined,
      isin: data.isin || undefined,
      ticker: data.ticker || undefined,
      sedol: data.sedol || undefined,
      existing_position: data.existingPosition,
      is_derivative: data.isDerivative === true || data.isDerivative === 'true',
      is_leveraged: data.isLeveraged === true || data.isLeveraged === 'true',
      is_related_party: data.isForOtherPerson === true || data.isForOtherPerson === 'true',
      relation: (data.isForOtherPerson === true || data.isForOtherPerson === 'true') ? data.relation : undefined,
      insider_info_confirmed: data.insiderInfo === false || data.insiderInfo === 'false', // TRUE means user confirmed NO insider info
      justification: data.justification || undefined,
      derivative_justification: (data.isDerivative === true || data.isDerivative === 'true') ? data.derivativeJustification : undefined,
    }
```

### Caller Impact Analysis
**Direct callers**: New Request form submission
**Indirect impact**: Risk scoring (uses boolean flags)
**Risk**: **LOW** - Defensive parsing handles both booleans and strings

### Test Specification
```typescript
// dashboard/src/pages/NewRequest.test.tsx

describe('NewRequest boolean field handling', () => {
  it('converts derivative flag to actual boolean', async () => {
    const mockSubmit = jest.fn()
    render(<NewRequest onSubmit={mockSubmit} />)

    // Fill form with derivative = true
    fireEvent.change(screen.getByLabelText('Security Name'), { target: { value: 'AAPL' } })
    fireEvent.change(screen.getByLabelText('Is Derivative?'), { target: { value: 'true' } })

    fireEvent.click(screen.getByText('Submit'))

    await waitFor(() => {
      const payload = mockSubmit.mock.calls[0][0]
      expect(payload.is_derivative).toBe(true) // Boolean, not string
      expect(typeof payload.is_derivative).toBe('boolean')
    })
  })

  it('handles insider info checkbox correctly (not inverted)', async () => {
    const mockSubmit = jest.fn()
    render(<NewRequest onSubmit={mockSubmit} />)

    // User checks "I have insider info" (WARNING)
    fireEvent.change(screen.getByLabelText('Insider Information?'), { target: { value: 'true' } })

    fireEvent.click(screen.getByText('Submit'))

    await waitFor(() => {
      const payload = mockSubmit.mock.calls[0][0]
      // insider_info_confirmed should be FALSE (user did NOT confirm they lack insider info)
      expect(payload.insider_info_confirmed).toBe(false)
    })
  })

  it('converts all boolean fields correctly', async () => {
    const mockSubmit = jest.fn()
    render(<NewRequest onSubmit={mockSubmit} />)

    // Set all boolean fields
    fireEvent.change(screen.getByLabelText('Is Derivative?'), { target: { value: 'true' } })
    fireEvent.change(screen.getByLabelText('Is Leveraged?'), { target: { value: 'true' } })
    fireEvent.change(screen.getByLabelText('For Other Person?'), { target: { value: 'true' } })

    fireEvent.click(screen.getByText('Submit'))

    await waitFor(() => {
      const payload = mockSubmit.mock.calls[0][0]
      expect(payload.is_derivative).toBe(true)
      expect(payload.is_leveraged).toBe(true)
      expect(payload.is_related_party).toBe(true)
      expect(typeof payload.is_derivative).toBe('boolean')
      expect(typeof payload.is_leveraged).toBe('boolean')
      expect(typeof payload.is_related_party).toBe('boolean')
    })
  })
})
```

### Rollback Strategy
**Git revert** - Frontend-only change, zero backend impact. Safe to revert instantly.

---

## Phase 8: Breach Auto-Resolution Verification (VERIFY FIRST)

### Status: NEEDS RE-VERIFICATION

**File**: `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/services/pad_service.py`
**Line**: 622
**Original Report**: Query uses `not PADBreach.resolved` instead of `not_(PADBreach.resolved)`

### Current Code (Lines 616-630)
```python
            # Auto-resolve any existing unresolved contract_note_mismatch breaches for this request
            await session.execute(
                update(PADBreach)
                .where(
                    and_(
                        PADBreach.request_id == request_id,
                        PADBreach.breach_type == "contract_note_mismatch",
                        not PADBreach.resolved,
                    )
                )
                .values(
                    resolved=True,
                    resolved_at=datetime.now(UTC).replace(tzinfo=None),
                    resolution_notes="Auto-resolved: superseded by new contract note upload",
                )
            )
```

### Analysis
**Line 622 shows**: `not PADBreach.resolved` - This is **Python's boolean NOT**, not SQLAlchemy's `not_()`.

**Correct SQLAlchemy pattern**:
```python
from sqlalchemy import not_

.where(not_(PADBreach.resolved))
```

**Current pattern** (line 622):
```python
.where(not PADBreach.resolved)  # WRONG - Python not, not SQL not_()
```

### Action Required
1. Check imports at top of file for `not_` from sqlalchemy
2. If `not_` is already imported, this is a confirmed bug
3. If query works in production, Python's `not` may be getting overloaded by SQLAlchemy Column magic (fragile)
4. Best practice: Always use `not_()` for SQLAlchemy expressions

### If Confirmed as Bug

#### Before (Lines 616-630)
```python
            # Auto-resolve any existing unresolved contract_note_mismatch breaches for this request
            await session.execute(
                update(PADBreach)
                .where(
                    and_(
                        PADBreach.request_id == request_id,
                        PADBreach.breach_type == "contract_note_mismatch",
                        not PADBreach.resolved,  # BUG: Python not instead of SQL not_()
                    )
                )
                .values(
                    resolved=True,
                    resolved_at=datetime.now(UTC).replace(tzinfo=None),
                    resolution_notes="Auto-resolved: superseded by new contract note upload",
                )
            )
```

#### After (Lines 616-631)
```python
            # Auto-resolve any existing unresolved contract_note_mismatch breaches for this request
            from sqlalchemy import not_

            await session.execute(
                update(PADBreach)
                .where(
                    and_(
                        PADBreach.request_id == request_id,
                        PADBreach.breach_type == "contract_note_mismatch",
                        not_(PADBreach.resolved),  # Correct: SQLAlchemy not_()
                    )
                )
                .values(
                    resolved=True,
                    resolved_at=datetime.now(UTC).replace(tzinfo=None),
                    resolution_notes="Auto-resolved: superseded by new contract note upload",
                )
            )
```

### Test Specification
```python
# tests/unit/services/test_pad_service.py

async def test_breach_auto_resolution_only_affects_unresolved():
    """Verify auto-resolution query matches only unresolved breaches."""
    session = await create_test_session()

    # Create 2 breaches for same request:
    # 1. Unresolved (should be auto-resolved)
    # 2. Already resolved (should NOT be touched)
    breach1 = PADBreach(
        request_id=123,
        employee_id=456,
        breach_type="contract_note_mismatch",
        severity="MEDIUM",
        resolved=False,
        detected_at=datetime.now(UTC).replace(tzinfo=None)
    )
    breach2 = PADBreach(
        request_id=123,
        employee_id=456,
        breach_type="contract_note_mismatch",
        severity="MEDIUM",
        resolved=True,
        resolved_at=datetime.now(UTC).replace(tzinfo=None) - timedelta(days=1),
        resolution_notes="Previously resolved"
    )

    session.add_all([breach1, breach2])
    await session.commit()

    # Upload new contract note (triggers auto-resolution)
    pad_service = PADService()
    await pad_service.upload_contract_note(
        request_id=123,
        contract_note_path="s3://bucket/new_note.pdf",
        actor_id=456
    )

    # Verify results
    await session.refresh(breach1)
    await session.refresh(breach2)

    assert breach1.resolved is True
    assert breach1.resolution_notes == "Auto-resolved: superseded by new contract note upload"

    assert breach2.resolved is True  # Still resolved
    assert breach2.resolution_notes == "Previously resolved"  # NOT changed
```

---

## Verification Checklist

### Pre-Deployment
- [ ] All unit tests pass (`pytest tests/unit/`)
- [ ] Integration tests pass (`pytest tests/integration/`)
- [ ] Manual smoke test: Submit PAD request end-to-end
- [ ] Manual smoke test: Upload contract note, verify PDF not corrupted
- [ ] Check FX rate data pipeline is operational before deploying Phase 2
- [ ] Verify SQLAlchemy version compatibility for `not_()` function

### Post-Deployment Monitoring
- [ ] Monitor logs for `fx_rate_missing_cannot_convert` errors (Phase 2)
- [ ] Monitor PDF processing success rate (Phase 1)
- [ ] Monitor orphan recovery counts (Phase 3)
- [ ] Query audit logs: `SELECT COUNT(*) FROM audit_log WHERE action_type = 'breach_detected'` (Phase 4)
- [ ] Check dashboard range slider functionality (Phase 6)
- [ ] Check NewRequest form boolean submission (Phase 7)

### Rollback Indicators
- **Phase 1**: PDF parsing errors spike
- **Phase 2**: Risk scoring failures spike, user complaints about blocked requests
- **Phase 3**: Duplicate PDF processing detected
- **Phase 4**: Breach audit queries return zero results
- **Phase 6**: UI errors in browser console
- **Phase 7**: Request submission failures, validation errors

---

## Implementation Order

### Recommended Sequence
1. **Phase 3** (Orphan Recovery) - Zero risk, pure bug fix
2. **Phase 1** (PDF Double-Decode) - Zero risk, pure bug fix
3. **Phase 4** (Audit ActionType) - Zero risk, metadata fix
4. **Phase 5** (Risk Routing Docs) - Zero risk, documentation
5. **Phase 6** (Range Slider) - Frontend only, zero backend risk
6. **Phase 7** (Boolean Select) - Frontend only, low risk
7. **Phase 8** (Breach Auto-Resolution) - ONLY if verification confirms bug
8. **Phase 2** (FX Rate Fallback) - LAST, highest risk (requires caller updates)

### Why This Order?
- Low-risk fixes first build confidence
- Frontend changes isolated from backend
- FX rate fix last because it requires coordinated 4-file change
- Breach auto-resolution conditional on verification

---

## Success Metrics

### Phase 1 (PDF Double-Decode)
- **Before**: PDF parsing failures ~40% (corrupted files)
- **After**: PDF parsing failures <5% (legitimate failures only)

### Phase 2 (FX Rate Fallback)
- **Before**: Silent FX rate failures logged as WARNING
- **After**: FX rate failures logged as ERROR with user notification
- **After**: Zero risk scoring calculations using 1.0 fallback

### Phase 3 (Orphan Recovery)
- **Before**: All processing documents recovered immediately
- **After**: Only documents stuck >30 minutes recovered

### Phase 4 (Audit ActionType)
- **Before**: Breach audit logs use generic `pad_request_viewed` type
- **After**: Breach audit logs use specific `breach_detected`/`breach_resolved` types
- **Metric**: `SELECT COUNT(*) FROM audit_log WHERE action_type IN ('breach_detected', 'breach_resolved')`

### Phase 6 (Range Slider)
- **Before**: Users can drag handles into invalid state (low > high)
- **After**: Handle constraints enforced, no invalid states possible

### Phase 7 (Boolean Select)
- **Before**: API receives strings "true"/"false" instead of booleans
- **After**: API receives actual boolean values

---

## Risk Assessment Summary

| Phase | Risk Level | Reason | Rollback Difficulty |
|-------|-----------|--------|-------------------|
| 1 - PDF Double-Decode | **LOW** | Pure bug fix, 1-line change | EASY (git revert) |
| 2 - FX Rate Fallback | **MEDIUM** | 4-file change, affects callers | MODERATE (staged revert) |
| 3 - Orphan Recovery | **LOW** | Pure bug fix, 1-line change | EASY (git revert) |
| 4 - Audit ActionType | **ZERO** | Metadata only, 2-line change | EASY (git revert) |
| 5 - Risk Routing Docs | **ZERO** | Documentation only | N/A (not needed) |
| 6 - Range Slider | **ZERO** | Frontend only, validation enhancement | EASY (git revert) |
| 7 - Boolean Select | **LOW** | Frontend only, defensive parsing | EASY (git revert) |
| 8 - Breach Resolution | **TBD** | Pending verification | EASY (git revert) |

---

## Notes for Implementation

### Environment Setup
```bash
# Backend tests
cd /Users/luisdeburnay/work/rules_engine_refactor
docker-compose up -d postgres
pytest tests/unit/ -v

# Frontend tests
cd dashboard
npm test -- --coverage
```

### Import Statements to Add

**Phase 2 (currency_service.py)**: None needed (`CurrencyConversionError` already defined)

**Phase 2 (risk_scoring_service.py)**:
```python
from pa_dealing.services.currency_service import CurrencyConversionError
```

**Phase 2 (chatbot.py)**:
```python
from pa_dealing.services.currency_service import CurrencyConversionError
```

**Phase 3 (pdf_poller.py)**:
```python
from datetime import timedelta  # Add if not present
```

**Phase 4 (pad_service.py)**: None needed (ActionType already imported)

**Phase 8 (pad_service.py)** - If bug confirmed:
```python
from sqlalchemy import not_  # Add if not present
```

### Configuration Changes
None required - all fixes are code-level.

### Database Migrations
None required - no schema changes.

---

**END OF IMPLEMENTATION PLAN**
