# Rubedo Performance Budgets & Hardware Constraints

## 1. Memory Budget Breakdown (Total Budget: $\le 40$ MB)

| Component | Target Memory Budget | Rationale |
|:---|:---|:---|
| **Base Rust Runtime** | $\approx 4\text{ MB}$ | Tokio worker threads + async executor baseline |
| **SQLite Pool (Crucible)** | $\approx 8\text{ MB}$ | In-process cache pages (`PRAGMA cache_size = -8000`) |
| **FastEmbed BGESmallENV15** | $\approx 22\text{ MB}$ | Quantized ONNX weights loaded into RAM |
| **LanceDB Vector Index** | $\approx 4\text{ MB}$ | Memory-mapped embedded vector index buffers |
| **Transient Overhead** | $\approx 2\text{ MB}$ | Working buffer allocations during step transitions |
| **Total Resident Memory (RSS)** | **$< 40\text{ MB}$** | **Ironclad Edge/Embedded Constraint** |

---

## 2. Latency SLAs

1. **Step Transition Checkpoint**: $< 5\text{ ms}$ SQLite disk sync.
2. **Suspended Step Resumption**: $< 10\text{ ms}$ from resume trigger to next step invocation.
3. **WASM Fuel Termination**: $< 15\text{ ms}$ upon infinite loop detection.
