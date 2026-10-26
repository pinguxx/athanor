//! Deterministic scoring metrics: ExactMatch, LevenshteinDistance, and JsonGrammarValidator.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::EvalsError;

/// Evaluation outcome emitted by deterministic scoring metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricScore {
    /// Normalized quality score in range [0.0, 1.0].
    pub score: f64,
    /// Indicates whether the evaluation meets or exceeds the required passing criteria.
    pub passed: bool,
    /// Optional diagnostic feedback, comparison details, or error reason.
    pub feedback: Option<String>,
}

impl MetricScore {
    /// Create a passing metric score with a 1.0 score and optional feedback.
    pub fn pass(feedback: Option<String>) -> Self {
        Self {
            score: 1.0,
            passed: true,
            feedback,
        }
    }

    /// Create a failing metric score with a 0.0 score and explanation.
    pub fn fail(feedback: impl Into<String>) -> Self {
        Self {
            score: 0.0,
            passed: false,
            feedback: Some(feedback.into()),
        }
    }

    /// Create a fractional metric score with explicit pass boolean.
    pub fn partial(score: f64, passed: bool, feedback: Option<String>) -> Self {
        Self {
            score: score.clamp(0.0, 1.0),
            passed,
            feedback,
        }
    }
}

// ---------------------------------------------------------------------------
// 1. ExactMatch
// ---------------------------------------------------------------------------

/// Verifies exact string equality with configurable whitespace and case tolerance.
#[derive(Debug, Clone)]
pub struct ExactMatch {
    /// When true, comparisons ignore character casing.
    pub ignore_case: bool,
    /// When true, trims leading and trailing whitespace before comparison.
    pub trim_whitespace: bool,
    /// When true, collapses consecutive whitespace characters into a single ASCII space.
    pub collapse_whitespace: bool,
}

impl Default for ExactMatch {
    fn default() -> Self {
        Self {
            ignore_case: false,
            trim_whitespace: true,
            collapse_whitespace: false,
        }
    }
}

impl ExactMatch {
    /// Construct a new `ExactMatch` validator with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set case insensitivity.
    pub fn ignore_case(mut self, ignore: bool) -> Self {
        self.ignore_case = ignore;
        self
    }

    /// Set whitespace trimming.
    pub fn trim_whitespace(mut self, trim: bool) -> Self {
        self.trim_whitespace = trim;
        self
    }

    /// Set whitespace collapsing.
    pub fn collapse_whitespace(mut self, collapse: bool) -> Self {
        self.collapse_whitespace = collapse;
        self
    }

    /// Evaluate string equality between actual completion and expected target.
    pub fn evaluate(&self, actual: &str, expected: &str) -> MetricScore {
        let norm_actual = self.normalize(actual);
        let norm_expected = self.normalize(expected);

        if norm_actual == norm_expected {
            MetricScore::pass(None)
        } else {
            MetricScore::fail(format!(
                "Exact match failed.\nExpected: \"{norm_expected}\"\nActual:   \"{norm_actual}\""
            ))
        }
    }

    fn normalize(&self, input: &str) -> String {
        let mut s = input.to_string();
        if self.trim_whitespace {
            s = s.trim().to_string();
        }
        if self.collapse_whitespace {
            s = s.split_whitespace().collect::<Vec<_>>().join(" ");
        }
        if self.ignore_case {
            s = s.to_lowercase();
        }
        s
    }
}

// ---------------------------------------------------------------------------
// 2. LevenshteinDistance
// ---------------------------------------------------------------------------

/// Calculates normalized Levenshtein edit distance similarity score in range [0.0, 1.0].
#[derive(Debug, Clone)]
pub struct LevenshteinDistance {
    /// Minimum similarity score required to qualify as a passing evaluation (default: 0.80).
    pub threshold: f64,
    /// When true, casing differences do not incur edit distance penalties.
    pub ignore_case: bool,
}

impl Default for LevenshteinDistance {
    fn default() -> Self {
        Self {
            threshold: 0.80,
            ignore_case: false,
        }
    }
}

impl LevenshteinDistance {
    /// Construct a new `LevenshteinDistance` evaluator with a passing threshold.
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
            ignore_case: false,
        }
    }

    /// Enable or disable case-insensitive comparison.
    pub fn ignore_case(mut self, ignore: bool) -> Self {
        self.ignore_case = ignore;
        self
    }

    /// Calculate the raw integer Levenshtein edit distance between two strings.
    pub fn raw_distance(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();

        let len_a = a_chars.len();
        let len_b = b_chars.len();

        if len_a == 0 {
            return len_b;
        }
        if len_b == 0 {
            return len_a;
        }

        let mut prev_row: Vec<usize> = (0..=len_b).collect();
        let mut curr_row: Vec<usize> = vec![0; len_b + 1];

        for i in 1..=len_a {
            curr_row[0] = i;
            for j in 1..=len_b {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };

                let deletion = prev_row[j] + 1;
                let insertion = curr_row[j - 1] + 1;
                let substitution = prev_row[j - 1] + cost;

                curr_row[j] = deletion.min(insertion).min(substitution);
            }
            prev_row.copy_from_slice(&curr_row);
        }

        prev_row[len_b]
    }

    /// Calculate normalized similarity score between two strings in range [0.0, 1.0].
    pub fn similarity(&self, actual: &str, expected: &str) -> f64 {
        let (a, b) = if self.ignore_case {
            (actual.to_lowercase(), expected.to_lowercase())
        } else {
            (actual.to_string(), expected.to_string())
        };

        let max_len = a.chars().count().max(b.chars().count());
        if max_len == 0 {
            return 1.0;
        }

        let dist = Self::raw_distance(&a, &b);
        1.0 - (dist as f64 / max_len as f64)
    }

    /// Evaluate similarity between actual output and expected target against threshold.
    pub fn evaluate(&self, actual: &str, expected: &str) -> MetricScore {
        let sim = self.similarity(actual, expected);
        let passed = sim >= self.threshold;

        let feedback = if passed {
            None
        } else {
            Some(format!(
                "Similarity score {sim:.3} below threshold {thresh:.3}",
                thresh = self.threshold
            ))
        };

        MetricScore::partial(sim, passed, feedback)
    }
}

// ---------------------------------------------------------------------------
// 3. JsonGrammarValidator
// ---------------------------------------------------------------------------

/// Validates that an agent completion conforms to valid JSON syntax and schema constraints.
#[derive(Debug, Clone, Default)]
pub struct JsonGrammarValidator {
    /// Top-level keys that must be present in the JSON object.
    pub required_properties: Vec<String>,
    /// Expected JSON root type ("object", "array", "string", "number", "boolean").
    pub expected_root_type: Option<String>,
    /// Optional JSON Schema specification value for structural verification.
    pub schema: Option<Value>,
}

impl JsonGrammarValidator {
    /// Construct a new `JsonGrammarValidator`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add required property names.
    pub fn with_required_properties(mut self, props: &[&str]) -> Self {
        self.required_properties = props.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Enforce an expected root JSON value type.
    pub fn with_expected_root_type(mut self, root_type: impl Into<String>) -> Self {
        self.expected_root_type = Some(root_type.into());
        self
    }

    /// Set an optional JSON Schema definition.
    pub fn with_schema(mut self, schema: Value) -> Self {
        self.schema = Some(schema);
        self
    }

    /// Validate the provided completion string.
    pub fn evaluate(&self, actual: &str) -> MetricScore {
        let parsed: Value = match serde_json::from_str(actual) {
            Ok(v) => v,
            Err(e) => {
                return MetricScore::fail(format!("JSON syntax error: {e}"));
            }
        };

        // 1. Verify root type if specified
        if let Some(expected_type) = &self.expected_root_type {
            let actual_type = json_type_name(&parsed);
            if actual_type != expected_type.as_str() {
                return MetricScore::fail(format!(
                    "Expected root JSON type '{expected_type}', found '{actual_type}'"
                ));
            }
        }

        // 2. Verify required properties on JSON object
        if !self.required_properties.is_empty() {
            let obj = match parsed.as_object() {
                Some(o) => o,
                None => {
                    return MetricScore::fail(
                        "Expected top-level JSON Object to verify required properties",
                    );
                }
            };

            for req in &self.required_properties {
                if !obj.contains_key(req) {
                    return MetricScore::fail(format!(
                        "Missing required property '{req}' in JSON payload"
                    ));
                }
            }
        }

        // 3. Verify against JSON Schema if provided
        if let Some(schema) = &self.schema {
            if let Err(err) = validate_against_schema(&parsed, schema) {
                return MetricScore::fail(err.to_string());
            }
        }

        MetricScore::pass(None)
    }
}

fn json_type_name(val: &Value) -> &'static str {
    match val {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn validate_against_schema(val: &Value, schema: &Value) -> Result<(), EvalsError> {
    let schema_obj = match schema.as_object() {
        Some(s) => s,
        None => return Ok(()),
    };

    // Check schema "type"
    if let Some(type_val) = schema_obj.get("type").and_then(|t| t.as_str()) {
        let actual_type = json_type_name(val);
        if type_val != actual_type {
            return Err(EvalsError::JsonGrammar(format!(
                "Type mismatch: expected '{type_val}', found '{actual_type}'"
            )));
        }
    }

    // Check schema "required" array
    if let Some(required) = schema_obj.get("required").and_then(|r| r.as_array()) {
        if let Some(obj) = val.as_object() {
            for req in required {
                if let Some(field) = req.as_str() {
                    if !obj.contains_key(field) {
                        return Err(EvalsError::JsonGrammar(format!(
                            "Missing required schema field: '{field}'"
                        )));
                    }
                }
            }
        }
    }

    // Check schema "properties"
    if let Some(properties) = schema_obj.get("properties").and_then(|p| p.as_object()) {
        if let Some(obj) = val.as_object() {
            for (prop_name, sub_schema) in properties {
                if let Some(field_val) = obj.get(prop_name) {
                    validate_against_schema(field_val, sub_schema)?;
                }
            }
        }
    }

    Ok(())
}
