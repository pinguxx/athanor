//! # Rubedo Evals
//!
//! Verification harness, deterministic metrics, and evaluation guardrails for the
//! Rubedo agent framework.
//!
//! Provides deterministic scoring metrics (`ExactMatch`, `LevenshteinDistance`,
//! `JsonGrammarValidator`), token consumption watchdog (`TokenBudgetGuard`), semantic
//! model-graded evaluations (`LlmJudge`), benchmark suites (`BenchmarkSuite`), and the
//! `#[artificer_test]` procedural macro for automated CI regression testing.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod error;
pub mod guardrails;
pub mod judge;
pub mod metrics;
pub mod suite;

pub use error::EvalsError;
pub use guardrails::{TokenBudgetConfig, TokenBudgetGuard, TokenUsageSnapshot};
pub use judge::{HeuristicJudgeEvaluator, JudgeCriteria, JudgeEvaluator, JudgeVerdict, LlmJudge};
pub use metrics::{ExactMatch, JsonGrammarValidator, LevenshteinDistance, MetricScore};
pub use rubedo_macros::artificer_test;
pub use suite::{ArtificerTestResult, BenchmarkSuite, EvaluationReport, TestCase, TestCaseResult};

/// Prelude module for convenient imports across evaluation suites.
pub mod prelude {
    pub use crate::error::EvalsError;
    pub use crate::guardrails::{TokenBudgetConfig, TokenBudgetGuard, TokenUsageSnapshot};
    pub use crate::judge::{
        HeuristicJudgeEvaluator, JudgeCriteria, JudgeEvaluator, JudgeVerdict, LlmJudge,
    };
    pub use crate::metrics::{
        ExactMatch, JsonGrammarValidator, LevenshteinDistance, MetricScore,
    };
    pub use crate::artificer_test;
    pub use crate::suite::{
        ArtificerTestResult, BenchmarkSuite, EvaluationReport, TestCase, TestCaseResult,
    };
}
