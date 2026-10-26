use athanor_core::reagent::{Reagent, ReagentError};
use athanor_macros::reagent;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Spec validation test: simple query tool.
#[reagent(name = "test_tool")]
async fn test_tool(
    /// The search query parameter.
    query: String,
) -> Result<String, ReagentError> {
    Ok(query)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Quote {
    symbol: String,
    price: f64,
    currency: String,
}

/// Fetches the latest stock quote for a given market symbol.
#[reagent(name = "get_stock_price")]
async fn get_stock_price(
    /// Ticker symbol (e.g. AAPL, NVDA)
    symbol: String,
    /// Currency code (default: USD)
    currency: Option<String>,
) -> Result<Quote, ReagentError> {
    if symbol.is_empty() {
        return Err(ReagentError::invalid_input("Ticker symbol cannot be empty"));
    }

    if symbol == "UNKNOWN" {
        return Err(ReagentError::recoverable_with_fix(
            "Ticker symbol was not recognized",
            "Use a valid listed equity ticker",
            json!({ "symbol": "AAPL", "currency": "USD" }),
        ));
    }

    let currency = currency.unwrap_or_else(|| "USD".to_string());
    Ok(Quote {
        symbol: symbol.to_uppercase(),
        price: 150.25,
        currency,
    })
}

/// Returns the status of the local alchemical reactor.
#[reagent(name = "reactor_status")]
async fn reactor_status() -> Result<String, ReagentError> {
    Ok("Reactor is at steady-state transmutation".into())
}

#[tokio::test]
async fn test_spec_validation_test_tool() {
    let tool = TestToolReagent;
    assert_eq!(tool.name(), "test_tool");

    let schema = tool.input_schema();
    let schema_str = schema.to_string();
    assert!(
        schema_str.contains("query"),
        "Schema should contain 'query' field: {}",
        schema_str
    );
    assert!(
        schema_str.contains("string"),
        "Field 'query' should be string type: {}",
        schema_str
    );

    let res = tool.transmute(json!({ "query": "alchemy" })).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), json!("alchemy"));
}

#[tokio::test]
async fn test_reagent_multi_params_and_docs() {
    let tool = GetStockPriceReagent;
    assert_eq!(tool.name(), "get_stock_price");
    assert!(
        tool.description()
            .contains("Fetches the latest stock quote"),
        "Description was: {}",
        tool.description()
    );

    let schema = tool.input_schema();
    let schema_str = schema.to_string();
    assert!(schema_str.contains("symbol"));
    assert!(schema_str.contains("currency"));

    // Success with both arguments
    let res = tool
        .transmute(json!({ "symbol": "nvda", "currency": "EUR" }))
        .await
        .expect("transmute should succeed");
    assert_eq!(
        res,
        json!({
            "symbol": "NVDA",
            "price": 150.25,
            "currency": "EUR"
        })
    );

    // Success with default/optional argument
    let res_default = tool
        .transmute(json!({ "symbol": "aapl" }))
        .await
        .expect("transmute should succeed with omitted optional parameter");
    assert_eq!(
        res_default,
        json!({
            "symbol": "AAPL",
            "price": 150.25,
            "currency": "USD"
        })
    );
}

#[tokio::test]
async fn test_reagent_zero_parameters() {
    let tool = ReactorStatusReagent;
    assert_eq!(tool.name(), "reactor_status");

    // Can invoke with empty object
    let res = tool
        .transmute(json!({}))
        .await
        .expect("zero-arg transmute should succeed with empty object");
    assert_eq!(res, json!("Reactor is at steady-state transmutation"));

    // Can invoke with null
    let res_null = tool
        .transmute(serde_json::Value::Null)
        .await
        .expect("zero-arg transmute should succeed with null");
    assert_eq!(res_null, json!("Reactor is at steady-state transmutation"));
}

#[tokio::test]
async fn test_reagent_error_propagation() {
    let tool = GetStockPriceReagent;

    // Invalid input from function logic
    let err_empty = tool.transmute(json!({ "symbol": "" })).await.unwrap_err();
    assert!(err_empty.is_invalid_input());

    // Recoverable error with guidance from function logic
    let err_unknown = tool
        .transmute(json!({ "symbol": "UNKNOWN" }))
        .await
        .unwrap_err();
    assert!(err_unknown.is_recoverable());
    if let ReagentError::Recoverable {
        guidance,
        suggested_fix,
        ..
    } = err_unknown
    {
        assert!(guidance.contains("valid listed equity"));
        assert_eq!(
            suggested_fix,
            Some(json!({ "symbol": "AAPL", "currency": "USD" }))
        );
    } else {
        panic!("Expected Recoverable error");
    }

    // Invalid schema input (passing integer where string is required)
    let err_type = tool
        .transmute(json!({ "symbol": 12345 }))
        .await
        .unwrap_err();
    assert!(err_type.is_invalid_input());
}

// Function with no doc comments and no macro attributes.
#[reagent]
async fn unannotated_calculator(x: i64, y: i64) -> Result<i64, ReagentError> {
    Ok(x + y)
}

/// Function with explicit description and custom crate override.
#[reagent(
    name = "overridden_tool",
    description = "Explicitly provided description override.",
    crate = "athanor_core"
)]
async fn sample_function(flag: bool) -> Result<String, ReagentError> {
    if flag {
        Ok("flag_true".into())
    } else {
        Err(ReagentError::fatal("flag_false"))
    }
}

#[tokio::test]
async fn test_reagent_default_attributes() {
    let tool = UnannotatedCalculatorReagent;
    // Default name is function name
    assert_eq!(tool.name(), "unannotated_calculator");
    assert_eq!(tool.description(), "");

    let out = tool
        .transmute(json!({ "x": 10, "y": 20 }))
        .await
        .expect("sum");
    assert_eq!(out, json!(30));

    let out_schema = tool.output_schema();
    assert_eq!(out_schema["type"], "object");
}

#[tokio::test]
async fn test_reagent_attribute_overrides() {
    let tool = SampleFunctionReagent;
    assert_eq!(tool.name(), "overridden_tool");
    assert_eq!(
        tool.description(),
        "Explicitly provided description override."
    );

    let ok_res = tool.transmute(json!({ "flag": true })).await.expect("ok");
    assert_eq!(ok_res, json!("flag_true"));

    let err_res = tool.transmute(json!({ "flag": false })).await.unwrap_err();
    assert!(err_res.is_fatal());
}

#[tokio::test]
async fn test_direct_async_function_call() {
    // The original function remains callable directly in normal Rust code
    let direct_res = get_stock_price("msft".into(), Some("GBP".into()))
        .await
        .expect("direct function call succeeded");

    assert_eq!(direct_res.symbol, "MSFT");
    assert_eq!(direct_res.currency, "GBP");
    assert_eq!(direct_res.price, 150.25);
}
