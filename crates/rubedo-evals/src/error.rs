//! Error types for the Rubedo evaluation harness and guardrails subsystem.

use thiserror::Error;

/// Central error enumeration for evaluation, guardrail, and benchmarking operations.
#[derive(Debug, Error)]
pub enum EvalsError {
    /// Failure during metric calculation or schema validation.
    #[error("Metric evaluation failed: {0}")]
    Metric(String),

    /// Syntax or structural grammar violation.
    #[error("JSON grammar validation failed: {0}")]
    JsonGrammar(String),

    /// Cumulative token consumption exceeded the configured budget.
    #[error("Token budget exceeded: used {used} tokens (prompt: {prompt_tokens}, completion: {completion_tokens}), limit was {limit}")]
    TokenBudgetExceeded {
        /// Number of prompt tokens ingested.
        prompt_tokens: u64,
        /// Number of completion tokens produced.
        completion_tokens: u64,
        /// Total tokens consumed.
        used: u64,
        /// Configured token budget ceiling.
        limit: u64,
    },

    /// Semantic model judge evaluation failure.
    #[error("LLM judge evaluation failed: {0}")]
    Judge(String),

    /// Benchmark test suite failed to satisfy the required pass threshold.
    #[error("Benchmark suite failed: pass rate {pass_rate:.2} below required threshold {threshold:.2} ({failed_cases}/{total_cases} cases failed)")]
    BenchmarkFailed {
        /// Achieved pass rate in range [0.0, 1.0].
        pass_rate: f64,
        /// Required threshold in range [0.0, 1.0].
        threshold: f64,
        /// Number of failed test cases.
        failed_cases: usize,
        /// Total number of test cases evaluated.
        total_cases: usize,
    },

    /// File I/O failure (e.g., loading test cases from disk).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
