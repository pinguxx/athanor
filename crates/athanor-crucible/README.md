# athanor-crucible

SQLite-backed event store, transactional checkpointing, and time-travel replay engine for the **Athanor** agentic framework.

## Features

- **SQLite WAL Concurrency**: High-throughput asynchronous persistence via `sqlx`.
- **Deterministic Checkpointing**: State snapshots saved before and after step executions.
- **Tokenized Resumption**: Atomic consumption of unforgeable UUIDv4 resumption tokens.
- **Event-Sourced Audit Trail**: Immutable log of step transitions (`vessel_events`) for compliance and debugging.
