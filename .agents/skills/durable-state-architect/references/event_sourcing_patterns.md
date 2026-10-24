# Event Sourcing & State Hydration Patterns in Rubedo

## 1. Dual-Store Model (Snapshots + Event Journal)
`rubedo-crucible` combines snapshots with append-only event logs:
- **`opus_runs.current_state`**: Provides fast $O(1)$ state retrieval when resuming or inspecting a run.
- **`vessel_events`**: Provides an immutable audit trail and enables step-by-step time-travel replay.

## 2. Replay Protocol
To replay an Opus run up to event index $k$:
1. Read initial state for `run_id`.
2. Select all `vessel_events` where `run_id = ?` ordered by `event_id ASC` up to $k$.
3. Sequentially fold each event payload into the replay context.
4. Verify the distilled state matches historical checkpoints.
