# athanor-opus

Durable directed acyclic graph (DAG) workflow engine with event-sourced persistence and checkpointing for the **Athanor** agentic framework.

## Features

- **Vessel Trait**: Execution nodes within an Opus capable of state transformation, output distillation, or execution suspension.
- **Durable Checkpointing**: Automatic state commits to SQLite (`athanor-crucible`) before and after each node transition.
- **Tokenized Suspension**: Seamless pause on human-in-the-loop approvals or external signals with unforgeable UUIDv4 tokens.
- **State Hydration & Resumption**: Resume suspended workflows from tokens with new payloads without state loss.
