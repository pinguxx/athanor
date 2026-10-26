# rubedo-evals

Verification harness, deterministic scoring metrics, and evaluation guardrails for the **Rubedo** agent framework.

## Overview

`rubedo-evals` provides:
- **Deterministic String & Schema Metrics**: `ExactMatch`, `LevenshteinDistance`, and `JsonGrammarValidator`.
- **Token Budget Watchdog**: `TokenBudgetGuard` preventing runaway token consumption.
- **Model-Graded Semantic Evaluator**: `LlmJudge` for scoring qualitative criteria (factual correctness, persona adherence, toxicity, tone).
- **Benchmark Suite & Runner**: `BenchmarkSuite` with threshold enforcement.
- **CI Test Macro**: `#[artificer_test]` running benchmark cases as native Cargo tests.
