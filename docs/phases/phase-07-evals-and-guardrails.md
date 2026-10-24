# Phase 7: Verification Harness & Evaluation Guardrails

## 1. Objectives & Overview
- **Target Crates**: `crates/rubedo-evals`
- **Primary Goal**: Provide a rigorous, automated evaluation and regression testing framework for Artificers. Includes deterministic string and grammar validators, token budget guardrails, an LLM-graded semantic judge, and the `#[artificer_test]` procedural macro to run benchmark suites in CI.

---

## 2. Detailed Task Breakdown

### Task 7.1: Scaffold `rubedo-evals`
- Create `crates/rubedo-evals/Cargo.toml` with dependencies:
  - `rubedo-core`, `serde`, `serde_json`, `tokio`, `thiserror`, `async-trait`, `tracing`

### Task 7.2: Deterministic Scoring Metrics
- In `crates/rubedo-evals/src/metrics.rs`:
  - `ExactMatch`: verifies string equality (ignoring whitespace if configured).
  - `LevenshteinDistance`: calculates normalized edit distance similarity score $[0.0, 1.0]$.
  - `JsonGrammarValidator`: verifies that agent completion adheres to a predefined JSON Schema structure without syntax violations.

### Task 7.3: Token Guardrails & Budget Watchdog
- In `crates/rubedo-evals/src/guardrails.rs`:
  - `TokenBudgetGuard`: tracks cumulative prompt and completion tokens.
  - Aborts execution if token consumption exceeds the configured budget limit.

### Task 7.4: Model-Graded Semantic Evaluator (`LlmJudge`)
- In `crates/rubedo-evals/src/judge.rs`:
  - Implement `LlmJudge`:
    - Evaluates qualitative criteria: factual correctness, persona adherence, absence of toxicity, tone.
    - Emits structured evaluation with numeric score $[1..5]$ and detailed justification.

### Task 7.5: Procedural Macro `#[artificer_test]`
- In `crates/rubedo-evals/src/macro_impl.rs`:
  - Macro attributes: `#[artificer_test(cases = "benchmarks/cases.json", threshold = 0.90)]`
  - Runs the target Artificer against all $N$ benchmark cases concurrently or sequentially.
  - Collects pass/fail ratios and token usage.
  - Fails test (panics) if pass rate falls below `threshold`.

---

## 3. Verification & Acceptance Criteria

### Integration Test
Create `crates/rubedo-evals/tests/eval_harness_test.rs`:
1. Define a benchmark suite of 5 mock test cases.
2. Run a mock Artificer where 2 out of 5 cases intentionally produce incorrect outputs.
3. Assert that when the pass threshold is set to $0.80$, the evaluation runner fails with a descriptive error report and non-zero exit code.
4. Assert that deterministic metrics (`ExactMatch`, `LevenshteinDistance`) produce correct mathematical scores.

### Execution Command
```bash
cargo test -p rubedo-evals
```
