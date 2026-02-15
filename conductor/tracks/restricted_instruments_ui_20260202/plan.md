# Plan: Restricted Instruments UI & Sync Controls

## Track Summary
Add restricted instruments table to Mako Conflicts page with sync controls and configurable frequency.

## Estimated Effort
**Total: 4-6 hours**

---

## Phase 1: Backend APIs (1.5 hours)

### Task 1.1: Add Restricted Instruments List Endpoint
**File:** `src/pa_dealing/api/routes/dashboard.py`

Add endpoint:
```python
@router.get("/restricted-instruments", response_model=APIResponse)
async def get_restricted_instruments(
    user: CurrentUserDep,
    include_inactive: bool = Query(False),
    search: str | None = Query(None),
):
    """Get list of restricted instruments."""
    _require_compliance_or_admin(user)

    async with get_session() as session:
        query = select(RestrictedSecurity)
        if not include_inactive:
            query = query.where(RestrictedSecurity.is_active == True)
        if search:
            query = query.where(
                or_(
                    RestrictedSecurity.inst_symbol.ilike(f"%{search}%"),
                    RestrictedSecurity.isin.ilike(f"%{search}%"),
                )
            )
        query = query.order_by(RestrictedSecurity.added_at.desc())

        result = await session.execute(query)
        instruments = result.scalars().all()

        return APIResponse(data=[
            {
                "id": i.id,
                "inst_symbol": i.inst_symbol,
                "isin": i.isin,
                "reason": i.reason,
                "is_active": i.is_active,
                "added_at": i.added_at.isoformat() if i.added_at else None,
            }
            for i in instruments
        ])
```

### Task 1.2: Add Manual Sync Trigger Endpoint
**File:** `src/pa_dealing/api/routes/config.py`

Add endpoint:
```python
@router.post("/restricted-list-sync", response_model=APIResponse)
async def trigger_restricted_list_sync(user: CurrentUserDep):
    """Manually trigger restricted list sync from Confluence."""
    await _require_compliance_role(user)

    from pa_dealing.services.restricted_list_sync import get_restricted_list_sync_service
    from pa_dealing.audit import get_audit_logger, ActionType, ActionStatus, ActorType

    service = get_restricted_list_sync_service()

    async with get_session() as session:
        try:
            stats = await service.sync_restricted_list(session)

            # Audit log
            audit = get_audit_logger()
            await audit.log(
                action_type=ActionType.CONFIG_CHANGE,
                action_status=ActionStatus.SUCCESS,
                actor_type=ActorType.USER,
                actor_identifier=user.email,
                entity_type="restricted_list_sync",
                details={"triggered_by": "manual", "stats": stats},
            )

            return APIResponse(
                message="Sync completed successfully",
                data={
                    "added": stats.get("added", 0),
                    "updated": stats.get("updated", 0),
                    "removed": stats.get("removed", 0),
                    "total": service._last_sync_count,
                }
            )
        except Exception as e:
            return APIResponse(
                success=False,
                message=f"Sync failed: {str(e)}",
                data=None
            )
```

### Task 1.3: Add Sync Interval Update Endpoint
**File:** `src/pa_dealing/api/routes/config.py`

Add endpoint:
```python
@router.put("/restricted-list-sync-interval", response_model=APIResponse)
async def update_sync_interval(
    user: CurrentUserDep,
    interval_minutes: int = Body(..., ge=15, le=1440),
):
    """Update the restricted list sync interval."""
    await _require_compliance_role(user)

    # Store in compliance_config
    async with get_session() as session:
        from sqlalchemy import update
        stmt = update(ComplianceConfig).where(
            ComplianceConfig.config_key == "restricted_list_sync_interval_minutes"
        ).values(config_value=str(interval_minutes))
        result = await session.execute(stmt)

        if result.rowcount == 0:
            # Insert if not exists
            new_config = ComplianceConfig(
                config_key="restricted_list_sync_interval_minutes",
                config_value=str(interval_minutes),
                config_type="int",
                description="Restricted list sync interval in minutes",
                category="sync",
            )
            session.add(new_config)

        await session.commit()

    # Reschedule job
    from pa_dealing.agents.monitoring.scheduler import get_scheduler
    scheduler = get_scheduler()
    scheduler.reschedule_restricted_list_sync(interval_minutes)

    return APIResponse(
        message=f"Sync interval updated to {interval_minutes} minutes",
        data={"interval_minutes": interval_minutes}
    )
```

### Task 1.4: Add Dynamic Interval Reloading to Scheduler
**File:** `src/pa_dealing/agents/monitoring/scheduler.py`

Add method:
```python
def reschedule_restricted_list_sync(self, interval_minutes: int) -> None:
    """Reschedule the restricted list sync job with new interval."""
    if not self.scheduler.running:
        logger.warning("Scheduler not running, cannot reschedule")
        return

    # Remove existing job
    try:
        self.scheduler.remove_job("restricted_list_sync")
    except Exception:
        pass

    # Add with new interval
    self.scheduler.add_job(
        lambda: self.monitoring_service.run_job(JobType.RESTRICTED_LIST_SYNC),
        IntervalTrigger(minutes=interval_minutes),
        id="restricted_list_sync",
        replace_existing=True,
        name="Restricted List Sync",
    )
    logger.info(f"Rescheduled restricted list sync to run every {interval_minutes} minutes")
```

---

## Phase 2: Frontend - Mako Conflicts Page (2 hours)

### Task 2.1: Add Types
**File:** `dashboard/src/types/index.ts`

```typescript
export interface RestrictedInstrument {
  id: number;
  inst_symbol: string;
  isin: string | null;
  reason: string | null;
  is_active: boolean;
  added_at: string | null;
}

export interface SyncStatus {
  status: 'success' | 'failure' | 'unknown' | 'pending';
  last_sync_time: string | null;
  count: number;
  error: string | null;
}
```

### Task 2.2: Add API Methods
**File:** `dashboard/src/api/client.ts`

```typescript
// In config object:
getRestrictedInstruments: async (params?: {
  include_inactive?: boolean;
  search?: string;
}): Promise<RestrictedInstrument[]> => {
  const response = await api.get('/dashboard/restricted-instruments', { params });
  return response.data;
},

triggerRestrictedListSync: async (): Promise<{
  added: number;
  updated: number;
  removed: number;
  total: number;
}> => {
  const response = await api.post('/config/restricted-list-sync');
  return response.data;
},

updateSyncInterval: async (intervalMinutes: number): Promise<void> => {
  await api.put('/config/restricted-list-sync-interval', { interval_minutes: intervalMinutes });
},
```

### Task 2.3: Create SyncStatusCard Component
**File:** `dashboard/src/components/SyncStatusCard.tsx`

Component with:
- Status badge (color-coded)
- Last sync timestamp
- Instrument count
- Sync frequency display
- "Sync Now" button with loading state

### Task 2.4: Create RestrictedInstrumentsSection Component
**File:** `dashboard/src/components/RestrictedInstrumentsSection.tsx`

Component with:
- SyncStatusCard at top
- Search input
- "Show Inactive" toggle
- Table with columns: Instrument, Reason, Added, Status
- Empty state

### Task 2.5: Integrate into MakoConflicts Page
**File:** `dashboard/src/pages/MakoConflicts.tsx`

- Add horizontal divider after existing table
- Add `<RestrictedInstrumentsSection />` below

---

## Phase 3: Frontend - Settings Page (0.5 hours)

### Task 3.1: Enhance Restricted List Sync Card
**File:** `dashboard/src/pages/Settings.tsx`

Update existing card:
- Add sync frequency dropdown (15/30/60/120/240 minutes)
- Add "Sync Now" button
- Wire up mutations

---

## Phase 4: Polish & Testing (1 hour)

### Task 4.1: Loading States
- Add loading skeletons to tables
- Add disabled states during sync

### Task 4.2: Error Handling
- Toast on sync failure
- Stale data warning (> 2 hours)

### Task 4.3: Tests
- Unit test for new endpoints
- Integration test for sync trigger

### Task 4.4: Manual UAT
- Verify all flows work end-to-end

---

## Dependencies

- Confluence sync service must be working
- APScheduler must be running

## Risks

| Risk | Mitigation |
|------|------------|
| Confluence unavailable | Graceful failure, show cached data |
| Large instrument list | Add pagination if > 100 |
| Scheduler not running | Check and warn in UI |

---

## Checklist

- [ ] Backend: GET /dashboard/restricted-instruments
- [ ] Backend: POST /config/restricted-list-sync
- [ ] Backend: PUT /config/restricted-list-sync-interval
- [ ] Backend: Dynamic scheduler reloading
- [ ] Frontend: TypeScript types
- [ ] Frontend: API client methods
- [ ] Frontend: SyncStatusCard component
- [ ] Frontend: RestrictedInstrumentsSection component
- [ ] Frontend: MakoConflicts page integration
- [ ] Frontend: Settings page sync controls
- [ ] Tests: Unit tests for endpoints
- [ ] Tests: Manual UAT
