use async_trait::async_trait;
use athanor_core::reagent::{Reagent, ReagentError};
use serde_json::{json, Value};

/// A test reagent that doubles an integer or errors on negative input.
struct DoublerReagent;

#[async_trait]
impl Reagent for DoublerReagent {
    fn name(&self) -> &'static str {
        "doubler"
    }

    fn description(&self) -> &'static str {
        "Multiplies an integer by two. Rejects negative numbers."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "number": { "type": "integer" }
            },
            "required": ["number"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "result": { "type": "integer" }
            }
        })
    }

    async fn transmute(&self, input: Value) -> Result<Value, ReagentError> {
        let num = input
            .get("number")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ReagentError::invalid_input("Missing or non-integer 'number' field"))?;

        if num < 0 {
            return Err(ReagentError::recoverable_with_fix(
                "Negative numbers are not allowed",
                "Provide an absolute or non-negative integer",
                json!({ "number": num.abs() }),
            ));
        }

        if num > 1_000_000 {
            return Err(ReagentError::fatal(
                "Number exceeds maximum allowable threshold",
            ));
        }

        Ok(json!({ "result": num * 2 }))
    }
}

#[tokio::test]
async fn test_reagent_contract_success() {
    let doubler = DoublerReagent;
    assert_eq!(doubler.name(), "doubler");
    assert!(doubler.description().contains("Multiplies an integer"));
    assert_eq!(doubler.input_schema()["required"][0], "number");

    let result = doubler.transmute(json!({ "number": 21 })).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!({ "result": 42 }));
}

#[tokio::test]
async fn test_reagent_invalid_input() {
    let doubler = DoublerReagent;
    let err = doubler.transmute(json!({})).await.unwrap_err();
    assert!(err.is_invalid_input());
    assert!(!err.is_recoverable());
    assert!(!err.is_fatal());
    assert!(err.to_string().contains("Missing or non-integer"));
}

#[tokio::test]
async fn test_reagent_recoverable_error() {
    let doubler = DoublerReagent;
    let err = doubler
        .transmute(json!({ "number": -5 }))
        .await
        .unwrap_err();
    assert!(err.is_recoverable());
    assert!(!err.is_fatal());
    assert!(!err.is_invalid_input());

    match err {
        ReagentError::Recoverable {
            message,
            guidance,
            suggested_fix,
        } => {
            assert_eq!(message, "Negative numbers are not allowed");
            assert!(guidance.contains("non-negative"));
            assert_eq!(suggested_fix, Some(json!({ "number": 5 })));
        }
        _ => panic!("Expected Recoverable error"),
    }
}

#[tokio::test]
async fn test_reagent_fatal_error() {
    let doubler = DoublerReagent;
    let err = doubler
        .transmute(json!({ "number": 2_000_000 }))
        .await
        .unwrap_err();
    assert!(err.is_fatal());
    assert!(!err.is_recoverable());
    assert!(!err.is_invalid_input());
    assert!(err.to_string().contains("maximum allowable threshold"));
}

#[test]
fn test_reagent_error_constructors_and_conversions() {
    let inv = ReagentError::invalid_input("bad format");
    assert!(inv.is_invalid_input());

    let rec = ReagentError::recoverable("try again", "check parameter type");
    assert!(rec.is_recoverable());
    assert!(rec.to_string().contains("Guidance: check parameter type"));

    let fatal = ReagentError::fatal("crash");
    assert!(fatal.is_fatal());

    let from_str: ReagentError = "static error".into();
    assert!(from_str.is_fatal());

    let from_string: ReagentError = String::from("dynamic error").into();
    assert!(from_string.is_fatal());

    let parse_err: Result<Value, _> = serde_json::from_str("{ bad json }");
    let from_serde: ReagentError = parse_err.unwrap_err().into();
    assert!(from_serde.is_invalid_input());
}
