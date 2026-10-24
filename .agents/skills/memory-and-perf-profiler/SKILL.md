---
name: memory-and-perf-profiler
description: >-
  Profiles resident set size (RSS) memory footprint and execution latency for Rubedo.
  Enforces the Tenet 5 sub-40MB memory SLA and benchmarks resume latency (<10ms).
  Use when validating memory consumption or profiling long-running agent workloads.
---

# Memory & Performance Profiler (Tenet 5 SLA Enforcer)

A specialized skill for tracking, auditing, and optimizing memory consumption and execution latency across the **Rubedo** agentic framework. Validates adherence to Core Tenet 5: **Zero External Infrastructure & Sub-40MB Resident Memory Footprint**.

---

## 🎯 SLA Performance Budgets

| Metric | Target SLA | Verification Method |
|:---|:---|:---|
| **Baseline Idle Memory (RSS)** | $< 40\text{ MB}$ | OS Process Working Set measurement |
| **Workflow Resumption Latency** | $< 10\text{ ms}$ | High-resolution timer on `OpusEngine::resume()` |
| **WASM Fuel Abort Latency** | $< 15\text{ ms}$ | Timer on `AlembicSandbox::execute_raw` fuel trap |
| **Reagent Transmute Overhead** | $< 1\text{ ms}$ | Benchmarked input deserialization & execution dispatch |
| **Checkpoint Commit Overhead** | $< 5\text{ KB}$ | SQLite DB disk space per step transition |

---

## 🔍 Profiling Methodology

### 1. Process Working Set (RSS) Measurement
On Windows, measured via PowerShell `Get-Process`:
```powershell
$proc = Get-Process -Name "my_rubedo_binary"
$rssMb = [math]::Round($proc.WorkingSet64 / 1MB, 2)
if ($rssMb -gt 40.0) {
    Write-Warning "SLA VIOLATION: Process RSS is ${rssMb}MB (Exceeds 40MB limit)"
}
```

### 2. High-Frequency Benchmarking
Use Rust's `criterion` or high-resolution `std::time::Instant` timers in integration benchmarks (`benches/`).

---

## 📋 Technical References
- [Performance Budgets & SLAs](./references/perf_budgets.md)
- Profiling Script: `profile_memory_rss.ps1`
- Resumption Benchmark Script: `bench_resume_latency.ps1`
