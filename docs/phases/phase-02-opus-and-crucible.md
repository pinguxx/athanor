# Phase 2: Durable Opus & Crucible Persistence

## 1. Objectives & Overview
- **Target Crates**: `crates/rubedo-crucible`, `crates/rubedo-opus`
- **Primary Goal**: Construct the SQLite-backed event store (`rubedo-crucible`) and the durable DAG workflow runtime (`rubedo-opus`). Workflows can execute sequential steps, suspend execution awaiting external tokens, persist snapshots to disk, and cleanly resume without data loss.

---

## 2. Detailed Task Breakdown

### Task 2.1: Scaffold `rubedo-crucible`
- Create `crates/rubedo-crucible/Cargo.toml` with dependencies:
  - `sqlx` with features `["runtime-tokio-rustls", "sqlite", "json"]`
  - `serde`, `serde_json`, `uuid`, `thiserror`, `tokio`, `tracing`

### Task 2.2: Implement SQLite Migrations
- Create `crates/rubedo-crucible/migrations/01_init.sql`:
  ```sql
  CREATE TABLE IF NOT EXISTS opus_runs (
      run_id TEXT PRIMARY KEY,
      opus_name TEXT NOT NULL,
      status TEXT NOT NULL,
      current_state TEXT NOT NULL, -- JSON
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
  );

  CREATE TABLE IF NOT EXISTS vessel_events (
      event_id INTEGER PRIMARY KEY AUTOINCREMENT,
      run_id TEXT NOT NULL,
      vessel_id TEXT NOT NULL,
      status TEXT NOT NULL,
      payload TEXT, -- JSON
      recorded_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY(run_id) REFERENCES opus_runs(run_id)
  );

  CREATE TABLE IF NOT EXISTS suspended_tokens (
      token TEXT PRIMARY KEY,
      run_id TEXT NOT NULL,
      vessel_id TEXT NOT NULL,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY(run_id) REFERENCES opus_runs(run_id)
  );
  ```

### Task 2.3: Implement `CrucibleEngine`
- In `crates/rubedo-crucible/src/lib.rs`:
  - `connect_in_memory()` and `connect_file(path: &str)`
  - Run embedded migrations automatically using `sqlx::migrate!()`.
  - Methods:
    - `insert_run(run_id: &str, opus_name: &str, initial_state: &Value) -> Result<(), CrucibleError>`
    - `update_run_state(run_id: &str, status: &str, current_state: &Value) -> Result<(), CrucibleError>`
    - `record_vessel_event(run_id: &str, vessel_id: &str, status: &str, payload: &Value) -> Result<(), CrucibleError>`
    - `register_suspended_token(token: &str, run_id: &str, vessel_id: &str) -> Result<(), CrucibleError>`
    - `consume_suspended_token(token: &str) -> Result<(String, String), CrucibleError>` (returns `(run_id, vessel_id)`)
    - `get_run(run_id: &str) -> Result<OpusRunRecord, CrucibleError>`

### Task 2.4: Scaffold `rubedo-opus`
- Create `crates/rubedo-opus/Cargo.toml` with dependencies:
  - `rubedo-core`, `rubedo-crucible`, `async-trait`, `serde`, `serde_json`, `uuid`, `thiserror`, `tokio`, `tracing`

### Task 2.5: Implement `Vessel` Trait & Status Model
- In `crates/rubedo-opus/src/vessel.rs`:
  ```rust
  use async_trait::async_trait;
  use serde::{Deserialize, Serialize};
  use serde_json::Value;

  #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
  pub enum VesselStatus {
      Pending,
      Active,
      Suspended { resume_token: String },
      Transmuted { output: Value },
      Corrupted { error: String },
  }

  #[async_trait]
  pub trait Vessel: Send + Sync + 'static {
      fn id(&self) -> &'static str;
      async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError>;
  }
  ```

### Task 2.6: Implement `OpusContext`, `OpusBuilder`, and `OpusEngine`
- `OpusContext`: Manages shared workflow state variables, execution journal, and resume inputs.
- `OpusBuilder`: Fluent builder to register `Vessel` nodes and transition edges.
- `OpusEngine`: Orchestrates step executions, calling `CrucibleEngine` before/after each node.
- Implement `OpusEngine::resume(run_id: &str, token: &str, payload: Value) -> Result<OpusResult, OpusError>`:
  - Validates and consumes the resume token from SQLite.
  - Hydrates state from SQLite.
  - Injects `payload` into `OpusContext`.
  - Resumes execution from the suspended Vessel to workflow completion.

---

## 3. Verification & Acceptance Criteria

### Integration Test
Create `crates/rubedo-opus/tests/durable_workflow_test.rs`:
1. Define a 3-step Opus:
   - Step 1: Computes initial value `{"stage": 1, "val": 42}`.
   - Step 2: Checks condition, yields `VesselStatus::Suspended { resume_token: uuid::Uuid::new_v4().to_string() }`.
   - Step 3: Adds the resume payload to stage 1 value and finishes.
2. Run the Opus:
   - Verify execution halts at Step 2 with `VesselStatus::Suspended`.
   - Verify SQLite contains the suspended token and run state.
3. Call `OpusEngine::resume(run_id, token, json!({"bonus": 10}))`:
   - Verify Step 3 executes.
   - Verify final state contains combined output.

### Execution Command
```bash
cargo test -p rubedo-crucible -p rubedo-opus
```
