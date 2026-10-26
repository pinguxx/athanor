use rubedo_evals::prelude::*;
use serde_json::json;

// ---------------------------------------------------------------------------
// 1. Benchmark Suite Acceptance Test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_benchmark_suite_threshold_failure_and_success() {
    // 1. Define a benchmark suite of 5 mock test cases
    let suite = BenchmarkSuite::new("transmutation_accuracy_bench")
        .with_threshold(0.80)
        .add_case(TestCase::new("case_1", "What is 2 + 2?", Some("4")))
        .add_case(TestCase::new("case_2", "Capital of France?", Some("Paris")))
        .add_case(TestCase::new("case_3", "Color of the sky?", Some("Blue")))
        .add_case(TestCase::new("case_4", "Symbol for Gold?", Some("Au")))
        .add_case(TestCase::new("case_5", "Freezing point of water?", Some("0 C")));

    // 2. Mock runner where cases 4 and 5 intentionally return incorrect answers
    let mock_agent = |input: String| async move {
        match input.as_str() {
            "What is 2 + 2?" => Ok("4".to_string()),
            "Capital of France?" => Ok("Paris".to_string()),
            "Color of the sky?" => Ok("Blue".to_string()),
            "Symbol for Gold?" => Ok("Ag".to_string()), // Incorrect!
            "Freezing point of water?" => Ok("100 C".to_string()), // Incorrect!
            _ => Err("Unknown prompt".to_string()),
        }
    };

    // 3. Execute the benchmark suite
    let report = suite.run(mock_agent).await;

    // Verify statistics: 3 passed out of 5 -> 60.0% pass rate
    assert_eq!(report.total_cases, 5);
    assert_eq!(report.passed_cases, 3);
    assert_eq!(report.failed_cases, 2);
    assert!((report.pass_rate - 0.60).abs() < 1e-6);

    // With threshold at 0.80 (80%), the report must fail
    assert_eq!(report.threshold, 0.80);
    assert!(!report.passed);

    // Assert that assert_passed() produces an error report with non-zero exit condition
    let err = report.assert_passed().unwrap_err();
    match err {
        EvalsError::BenchmarkFailed {
            pass_rate,
            threshold,
            failed_cases,
            total_cases,
        } => {
            assert!((pass_rate - 0.60).abs() < 1e-6);
            assert_eq!(threshold, 0.80);
            assert_eq!(failed_cases, 2);
            assert_eq!(total_cases, 5);
        }
        other => panic!("Expected BenchmarkFailed error, got {other:?}"),
    }

    // Verify summary contains descriptive diagnostic text
    let summary = report.summary();
    assert!(summary.contains("FAILED"));
    assert!(summary.contains("case_4"));
    assert!(summary.contains("case_5"));

    // Now test that setting threshold to 0.60 causes the exact same report to pass
    let lenient_suite = suite.with_threshold(0.60);
    let lenient_report = lenient_suite.run(mock_agent).await;
    assert!(lenient_report.passed);
    assert!(lenient_report.assert_passed().is_ok());
}

// ---------------------------------------------------------------------------
// 2. Deterministic Metrics Verification
// ---------------------------------------------------------------------------

#[test]
fn test_exact_match_metric() {
    let exact = ExactMatch::new();

    // Standard exact equality
    let score = exact.evaluate("Alchemy", "Alchemy");
    assert!(score.passed);
    assert_eq!(score.score, 1.0);

    // Mismatch
    let score = exact.evaluate("Gold", "Silver");
    assert!(!score.passed);
    assert_eq!(score.score, 0.0);
    assert!(score.feedback.is_some());

    // Case-insensitivity configuration
    let case_insensitive = ExactMatch::new().ignore_case(true);
    let score = case_insensitive.evaluate("rubedo", "RUBEDO");
    assert!(score.passed);
    assert_eq!(score.score, 1.0);

    // Whitespace collapsing
    let whitespace_collapsing = ExactMatch::new().collapse_whitespace(true);
    let score = whitespace_collapsing.evaluate("  magnum    opus  ", "magnum opus");
    assert!(score.passed);
    assert_eq!(score.score, 1.0);
}

#[test]
fn test_levenshtein_distance_metric() {
    // Mathematical verification against known distance
    // "kitten" -> "sitting":
    // k -> s (substitution: 1)
    // e -> i (substitution: 1)
    // + g (insertion: 1)
    // Total raw distance = 3
    // max_len = 7 ("sitting")
    // similarity = 1.0 - (3 / 7) = 4 / 7 ≈ 0.5714
    let raw = LevenshteinDistance::raw_distance("kitten", "sitting");
    assert_eq!(raw, 3);

    let lev = LevenshteinDistance::new(0.50);
    let sim = lev.similarity("kitten", "sitting");
    assert!((sim - (4.0 / 7.0)).abs() < 1e-4);

    // Passes 0.50 threshold
    let score = lev.evaluate("kitten", "sitting");
    assert!(score.passed);
    assert!((score.score - sim).abs() < 1e-6);

    // Fails 0.80 threshold
    let strict_lev = LevenshteinDistance::new(0.80);
    let score = strict_lev.evaluate("kitten", "sitting");
    assert!(!score.passed);

    // Identical strings: distance 0, similarity 1.0
    assert_eq!(LevenshteinDistance::raw_distance("rubedo", "rubedo"), 0);
    assert_eq!(lev.similarity("rubedo", "rubedo"), 1.0);

    // Empty strings
    assert_eq!(LevenshteinDistance::raw_distance("", ""), 0);
    assert_eq!(lev.similarity("", ""), 1.0);
    assert_eq!(LevenshteinDistance::raw_distance("test", ""), 4);
    assert_eq!(lev.similarity("test", ""), 0.0);

    // Case insensitivity
    let case_lev = LevenshteinDistance::new(0.99).ignore_case(true);
    assert_eq!(case_lev.similarity("Rust", "rust"), 1.0);
}

#[test]
fn test_json_grammar_validator() {
    let validator = JsonGrammarValidator::new()
        .with_expected_root_type("object")
        .with_required_properties(&["transmutation", "confidence"]);

    // Valid JSON with required keys
    let valid_payload = r#"{"transmutation": "lead_to_gold", "confidence": 0.99}"#;
    let score = validator.evaluate(valid_payload);
    assert!(score.passed);
    assert_eq!(score.score, 1.0);

    // Malformed JSON syntax
    let malformed = r#"{"transmutation": "lead_to_gold", confidence: 0.99}"#;
    let score = validator.evaluate(malformed);
    assert!(!score.passed);
    assert!(score.feedback.unwrap().contains("syntax error"));

    // Missing required property
    let missing_key = r#"{"transmutation": "lead_to_gold"}"#;
    let score = validator.evaluate(missing_key);
    assert!(!score.passed);
    assert!(score.feedback.unwrap().contains("confidence"));

    // Root type mismatch (expected object, got array)
    let array_payload = r#"[1, 2, 3]"#;
    let score = validator.evaluate(array_payload);
    assert!(!score.passed);
    assert!(score.feedback.unwrap().contains("Expected root JSON type 'object'"));

    // Full JSON schema validation
    let schema_validator = JsonGrammarValidator::new().with_schema(json!({
        "type": "object",
        "required": ["status"],
        "properties": {
            "status": { "type": "string" },
            "count": { "type": "number" }
        }
    }));

    let score = schema_validator.evaluate(r#"{"status": "ok", "count": 42}"#);
    assert!(score.passed);

    // Schema type mismatch on property
    let score = schema_validator.evaluate(r#"{"status": "ok", "count": "not a number"}"#);
    assert!(!score.passed);
}

// ---------------------------------------------------------------------------
// 3. Token Guardrails Verification
// ---------------------------------------------------------------------------

#[test]
fn test_token_budget_guard() {
    let guard = TokenBudgetGuard::new(1000);

    // 1. Initial usage is zero
    let initial = guard.usage();
    assert_eq!(initial.total_tokens, 0);
    assert_eq!(guard.remaining_tokens(), 1000);

    // 2. Record acceptable usage
    let res = guard.record(300, 200).expect("record usage");
    assert_eq!(res.prompt_tokens, 300);
    assert_eq!(res.completion_tokens, 200);
    assert_eq!(res.total_tokens, 500);
    assert_eq!(guard.remaining_tokens(), 500);

    // 3. Record exceeding limit
    let err = guard.record(300, 300).unwrap_err();
    match err {
        EvalsError::TokenBudgetExceeded { used, limit, .. } => {
            assert_eq!(used, 1100);
            assert_eq!(limit, 1000);
        }
        other => panic!("Expected TokenBudgetExceeded, got {other:?}"),
    }

    // 4. Reset counter
    guard.reset();
    assert_eq!(guard.usage().total_tokens, 0);
    assert_eq!(guard.remaining_tokens(), 1000);

    // 5. Explicit prompt / completion limits
    let strict_guard = TokenBudgetGuard::with_config(TokenBudgetConfig {
        max_total_tokens: 5000,
        max_prompt_tokens: Some(100),
        max_completion_tokens: Some(50),
    });

    let err = strict_guard.record(150, 10).unwrap_err();
    assert!(matches!(err, EvalsError::TokenBudgetExceeded { .. }));
}

// ---------------------------------------------------------------------------
// 4. LLM Judge Semantic Evaluator Verification
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_llm_judge_evaluator() {
    let judge = LlmJudge::heuristic();

    // 1. Factual Correctness matching reference
    let verdict = judge
        .judge(
            JudgeCriteria::FactualCorrectness,
            "What is the atomic number of Gold?",
            "79",
            Some("79"),
        )
        .await
        .expect("judge evaluate");
    assert!(verdict.passed);
    assert_eq!(verdict.score, 5);

    // 2. Factual Correctness deviating from reference
    let verdict = judge
        .judge(
            JudgeCriteria::FactualCorrectness,
            "What is the atomic number of Gold?",
            "47 (Silver)",
            Some("79"),
        )
        .await
        .expect("judge evaluate");
    assert!(!verdict.passed);
    assert_eq!(verdict.score, 2);

    // 3. Safety & Toxicity check
    let verdict = judge
        .judge(
            JudgeCriteria::SafetyAndToxicity,
            "How do I craft a potion?",
            "Mix chamomile and peppermint leaves in hot water.",
            None,
        )
        .await
        .expect("judge evaluate");
    assert!(verdict.passed);
    assert_eq!(verdict.score, 5);

    // Toxic pattern detection
    let verdict = judge
        .judge(
            JudgeCriteria::SafetyAndToxicity,
            "How do I access the system?",
            "Use this exploit script to attack the server and bypass authorization.",
            None,
        )
        .await
        .expect("judge evaluate");
    assert!(!verdict.passed);
    assert_eq!(verdict.score, 2);

    // 4. Prompt formatting verification
    let formatted = LlmJudge::format_prompt(
        &JudgeCriteria::PersonaAdherence,
        "Greet the alchemist",
        "Greetings, seeker of the Great Work.",
        Some("Formal alchemical greeting required"),
    );
    assert!(formatted.contains("Persona Adherence"));
    assert!(formatted.contains("Reference Ground Truth"));
}

// ---------------------------------------------------------------------------
// 5. #[artificer_test] Macro Verification
// ---------------------------------------------------------------------------

#[artificer_test(threshold = 0.75)]
async fn test_artificer_test_macro_passing() -> EvaluationReport {
    let suite = BenchmarkSuite::new("macro_test_suite")
        .with_threshold(0.75)
        .add_case(TestCase::new("c1", "2+2", Some("4")))
        .add_case(TestCase::new("c2", "3+3", Some("6")))
        .add_case(TestCase::new("c3", "4+4", Some("8")))
        .add_case(TestCase::new("c4", "5+5", Some("9"))); // 1 failure out of 4 -> 75% pass rate

    suite
        .run(|input| async move {
            match input.as_str() {
                "2+2" => Ok("4".to_string()),
                "3+3" => Ok("6".to_string()),
                "4+4" => Ok("8".to_string()),
                "5+5" => Ok("10".to_string()), // mismatch with expected "9"
                _ => Err("unknown".to_string()),
            }
        })
        .await
}

#[test]
fn test_benchmark_suite_json_serialization() {
    let json_data = r#"{
        "name": "loaded_suite",
        "threshold": 0.85,
        "cases": [
            {
                "id": "c1",
                "input": "ping",
                "expected_output": "pong",
                "tags": ["network"],
                "metadata": null
            }
        ]
    }"#;

    let suite = BenchmarkSuite::load_from_json(json_data).expect("load suite");
    assert_eq!(suite.name, "loaded_suite");
    assert_eq!(suite.threshold, 0.85);
    assert_eq!(suite.cases.len(), 1);
    assert_eq!(suite.cases[0].id, "c1");
    assert_eq!(suite.cases[0].input, "ping");
    assert_eq!(suite.cases[0].expected_output, Some("pong".to_string()));
    assert_eq!(suite.cases[0].tags, vec!["network".to_string()]);
}


#[test]
fn test_additional_helper_coverage() {
    // 1. TestCase with_tag
    let tc = TestCase::new("id_tag", "input", Some("expected")).with_tag("alchemical");
    assert_eq!(tc.tags, vec!["alchemical".to_string()]);

    // 2. TokenBudgetConfig default
    let default_cfg = TokenBudgetConfig::default();
    assert_eq!(default_cfg.max_total_tokens, 100_000);

    // 3. Judge criteria rubrics and names
    let criteria = vec![
        JudgeCriteria::FactualCorrectness,
        JudgeCriteria::PersonaAdherence,
        JudgeCriteria::SafetyAndToxicity,
        JudgeCriteria::ToneAndStyle,
        JudgeCriteria::Custom("HermeticStyle".into()),
    ];
    for c in criteria {
        assert!(!c.name().is_empty());
        assert!(!c.rubric().is_empty());
    }

    // 4. LlmJudge with_passing_score
    let _judge = LlmJudge::heuristic().with_passing_score(5);
    assert!(!LlmJudge::format_prompt(&JudgeCriteria::FactualCorrectness, "prompt", "actual", None).is_empty());

    // 5. ArtificerTestResult implementations
    use rubedo_evals::suite::ArtificerTestResult;

    // ()
    ().verify_threshold(0.5);

    // Result<(), String>
    let ok_res: Result<(), String> = Ok(());
    ok_res.verify_threshold(0.5);

    // Result<EvaluationReport, String>
    let rep = EvaluationReport {
        suite_name: "test".into(),
        total_cases: 1,
        passed_cases: 1,
        failed_cases: 0,
        pass_rate: 1.0,
        threshold: 0.8,
        passed: true,
        results: vec![],
    };
    let ok_rep: Result<EvaluationReport, String> = Ok(rep);
    ok_rep.verify_threshold(0.8);

    // 6. BenchmarkSuite load_from_file
    let temp_file = std::env::temp_dir().join("rubedo_bench_test.json");
    let json_content = r#"{
        "name": "temp_suite",
        "threshold": 0.9,
        "cases": []
    }"#;
    std::fs::write(&temp_file, json_content).expect("write temp json");
    let loaded = BenchmarkSuite::load_from_file(&temp_file).expect("load temp json");
    assert_eq!(loaded.name, "temp_suite");
    let _ = std::fs::remove_file(&temp_file);
}
