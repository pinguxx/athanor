//! Benchmark test suite and evaluation runner primitives.

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::EvalsError;
use crate::metrics::{ExactMatch, MetricScore};

/// A single benchmark test case.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCase {
    /// Unique identifier for this test case.
    pub id: String,
    /// User prompt or query passed into the agent.
    pub input: String,
    /// Expected canonical target completion.
    pub expected_output: Option<String>,
    /// Categorization tags (e.g. "math", "safety", "reasoning").
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional metadata payload.
    #[serde(default)]
    pub metadata: Value,
}

impl TestCase {
    /// Construct a basic test case with id, input, and expected output.
    pub fn new(
        id: impl Into<String>,
        input: impl Into<String>,
        expected: Option<impl Into<String>>,
    ) -> Self {
        Self {
            id: id.into(),
            input: input.into(),
            expected_output: expected.map(|e| e.into()),
            tags: Vec::new(),
            metadata: Value::Null,
        }
    }

    /// Add a tag to this test case.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Execution outcome for a single evaluated test case.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCaseResult {
    /// Associated test case identifier.
    pub case_id: String,
    /// Indicates whether the case satisfied the passing metric criteria.
    pub passed: bool,
    /// Quantitative score assigned to the output [0.0, 1.0].
    pub score: f64,
    /// The actual string produced by the evaluated agent.
    pub actual_output: String,
    /// Diagnostic failure message if the case did not pass.
    pub error_message: Option<String>,
    /// Estimated or recorded token count consumed during execution.
    pub tokens_used: Option<u64>,
}

/// Comprehensive evaluation report summarizing benchmark results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationReport {
    /// Name or title of the benchmark suite.
    pub suite_name: String,
    /// Total number of evaluated cases.
    pub total_cases: usize,
    /// Number of cases successfully passed.
    pub passed_cases: usize,
    /// Number of failed cases.
    pub failed_cases: usize,
    /// Achieved pass rate in range [0.0, 1.0].
    pub pass_rate: f64,
    /// Target pass threshold in range [0.0, 1.0].
    pub threshold: f64,
    /// Overall pass/fail determination (`pass_rate >= threshold`).
    pub passed: bool,
    /// Individual outcomes for each test case.
    pub results: Vec<TestCaseResult>,
}

impl EvaluationReport {
    /// Generate a human-readable summary table for console output or CI logs.
    pub fn summary(&self) -> String {
        let status = if self.passed { "PASSED" } else { "FAILED" };
        let mut out = format!(
            "==========================================================\n\
             EVALUATION REPORT: {name} [{status}]\n\
             ==========================================================\n\
             Total Cases:  {total}\n\
             Passed:       {passed} ({rate:.1}%)\n\
             Failed:       {failed}\n\
             Threshold:    {thresh:.1}%\n\
             ----------------------------------------------------------\n",
            name = self.suite_name,
            status = status,
            total = self.total_cases,
            passed = self.passed_cases,
            rate = self.pass_rate * 100.0,
            failed = self.failed_cases,
            thresh = self.threshold * 100.0
        );

        for res in &self.results {
            let mark = if res.passed { "✓" } else { "✗" };
            out.push_str(&format!(
                "  [{mark}] {id} (score: {score:.2})\n",
                id = res.case_id,
                score = res.score
            ));
            if let Some(err) = &res.error_message {
                out.push_str(&format!("      Reason: {err}\n"));
            }
        }
        out.push_str("==========================================================");
        out
    }

    /// Assert that the evaluation report passed; returns an `Err(EvalsError)` otherwise.
    pub fn assert_passed(&self) -> Result<(), EvalsError> {
        if self.passed {
            Ok(())
        } else {
            Err(EvalsError::BenchmarkFailed {
                pass_rate: self.pass_rate,
                threshold: self.threshold,
                failed_cases: self.failed_cases,
                total_cases: self.total_cases,
            })
        }
    }
}

/// A structured collection of test cases with a passing score threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    /// Title or identifier of the suite.
    pub name: String,
    /// Minimum required pass rate in range [0.0, 1.0] (default: 0.90).
    pub threshold: f64,
    /// Test cases composing the benchmark.
    pub cases: Vec<TestCase>,
}

impl BenchmarkSuite {
    /// Construct a new empty benchmark suite.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            threshold: 0.90,
            cases: Vec::new(),
        }
    }

    /// Set the required pass rate threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Append a test case to the suite.
    pub fn add_case(mut self, case: TestCase) -> Self {
        self.cases.push(case);
        self
    }

    /// Load benchmark cases from a JSON string.
    pub fn load_from_json(json_str: &str) -> Result<Self, EvalsError> {
        let suite: BenchmarkSuite = serde_json::from_str(json_str)?;
        Ok(suite)
    }

    /// Load benchmark cases from a JSON file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, EvalsError> {
        let content = fs::read_to_string(path)?;
        Self::load_from_json(&content)
    }

    /// Run the benchmark suite against an asynchronous evaluation function.
    pub async fn run<F, Fut>(&self, runner: F) -> EvaluationReport
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<String, String>>,
    {
        let exact = ExactMatch::default();
        let mut results = Vec::with_capacity(self.cases.len());

        for case in &self.cases {
            let actual_res = runner(case.input.clone()).await;

            let (actual_output, metric_score) = match actual_res {
                Ok(output) => {
                    let score = if let Some(expected) = &case.expected_output {
                        exact.evaluate(&output, expected)
                    } else {
                        // Without reference, non-empty is considered a pass
                        if output.trim().is_empty() {
                            MetricScore::fail("Output was empty")
                        } else {
                            MetricScore::pass(None)
                        }
                    };
                    (output, score)
                }
                Err(err) => (
                    String::new(),
                    MetricScore::fail(format!("Execution error: {err}")),
                ),
            };

            results.push(TestCaseResult {
                case_id: case.id.clone(),
                passed: metric_score.passed,
                score: metric_score.score,
                actual_output,
                error_message: metric_score.feedback,
                tokens_used: None,
            });
        }

        let total_cases = results.len();
        let passed_cases = results.iter().filter(|r| r.passed).count();
        let failed_cases = total_cases.saturating_sub(passed_cases);
        let pass_rate = if total_cases > 0 {
            passed_cases as f64 / total_cases as f64
        } else {
            1.0
        };

        let passed = pass_rate >= self.threshold;

        EvaluationReport {
            suite_name: self.name.clone(),
            total_cases,
            passed_cases,
            failed_cases,
            pass_rate,
            threshold: self.threshold,
            passed,
            results,
        }
    }
}


/// Helper trait allowing `#[artificer_test]` to inspect test return values and enforce thresholds.
pub trait ArtificerTestResult {
    /// Verify that the return value meets or exceeds the required threshold.
    fn verify_threshold(self, threshold: f64);
}

impl ArtificerTestResult for EvaluationReport {
    fn verify_threshold(self, threshold: f64) {
        if self.pass_rate < threshold || !self.passed {
            panic!(
                "Evaluation benchmark failed!\nPass rate: {:.1}% (required: {:.1}%)\n\n{}",
                self.pass_rate * 100.0,
                threshold * 100.0,
                self.summary()
            );
        }
    }
}

impl<E: std::fmt::Display> ArtificerTestResult for Result<EvaluationReport, E> {
    fn verify_threshold(self, threshold: f64) {
        match self {
            Ok(report) => report.verify_threshold(threshold),
            Err(e) => panic!("Evaluation benchmark returned error: {e}"),
        }
    }
}

impl ArtificerTestResult for () {
    fn verify_threshold(self, _threshold: f64) {}
}

impl<E: std::fmt::Display> ArtificerTestResult for Result<(), E> {
    fn verify_threshold(self, _threshold: f64) {
        if let Err(e) = self {
            panic!("Test returned error: {e}");
        }
    }
}
