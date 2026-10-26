# Crucible SQLite Schema Specification

This document details the exact SQLite DDL schema used by `athanor-crucible` (`migrations/01_init.sql`).

---

## 1. DDL Statements

```sql
-- Track overall Opus run lifecycle and state snapshots
CREATE TABLE IF NOT EXISTS opus_runs (
    run_id TEXT PRIMARY KEY,
    opus_name TEXT NOT NULL,
    status TEXT NOT NULL, -- 'Pending', 'Active', 'Suspended', 'Completed', 'Failed'
    current_state TEXT NOT NULL, -- JSON snapshot of shared workflow memory
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Append-only journal of step-by-step Vessel transitions
CREATE TABLE IF NOT EXISTS vessel_events (
    event_id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    vessel_id TEXT NOT NULL,
    status TEXT NOT NULL, -- 'Active', 'Transmuted', 'Suspended', 'Corrupted'
    payload TEXT, -- JSON output or error message
    recorded_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(run_id) REFERENCES opus_runs(run_id)
);

-- Active unconsumed resumption tokens
CREATE TABLE IF NOT EXISTS suspended_tokens (
    token TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    vessel_id TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(run_id) REFERENCES opus_runs(run_id)
);

-- Performance Indexes
CREATE INDEX IF NOT EXISTS idx_vessel_events_run ON vessel_events(run_id);
CREATE INDEX IF NOT EXISTS idx_suspended_tokens_run ON suspended_tokens(run_id);
```

---

## 2. Invariants for `CrucibleEngine`
1. **Foreign Key Enforcement**: `PRAGMA foreign_keys = ON;` must be executed upon opening any SQLite connection.
2. **Atomic Resumption**:
   ```sql
   DELETE FROM suspended_tokens WHERE token = ? RETURNING run_id, vessel_id;
   ```
   If no rows are returned, resumption MUST fail with `CrucibleError::TokenNotFound`.
