//! # Example: Basic Artificer
//!
//! Demonstrates constructing an autonomous [`Artificer`] equipped with executable
//! capabilities declared via the [`#[reagent]`](rubedo_macros::reagent) procedural macro.
//!
//! Simulates a multi-step ReAct (Reasoning + Acting) loop where the agent inspects
//! tool schemas, invokes capabilities with validated JSON arguments, and synthesizes
//! a verified final answer.

use std::error::Error;
use std::sync::Arc;

use rubedo_core::artificer::Artificer;
use rubedo_core::reagent::ReagentError;
use rubedo_macros::reagent;
use serde_json::json;

// ---------------------------------------------------------------------------
// 1. Tool Declarations via #[reagent]
// ---------------------------------------------------------------------------

/// Mathematical calculator performing arithmetic operations.
#[reagent(name = "calculator")]
async fn calculate(
    /// Mathematical operation to perform ("add", "subtract", "multiply", "divide")
    operation: String,
    /// First numeric operand
    a: f64,
    /// Second numeric operand
    b: f64,
) -> Result<f64, ReagentError> {
    match operation.to_lowercase().as_str() {
        "add" => Ok(a + b),
        "subtract" => Ok(a - b),
        "multiply" => Ok(a * b),
        "divide" => {
            if b.abs() < f64::EPSILON {
                Err(ReagentError::recoverable(
                    "Division by zero is undefined",
                    "Provide a non-zero denominator operand 'b'",
                ))
            } else {
                Ok(a / b)
            }
        }
        unknown => Err(ReagentError::invalid_input(format!(
            "Unsupported operation '{unknown}'. Supported: add, subtract, multiply, divide"
        ))),
    }
}

/// Meteorological weather lookup service for global cities.
#[reagent(name = "fetch_weather")]
async fn fetch_weather(
    /// Name of the city to look up weather conditions for
    city: String,
) -> Result<serde_json::Value, ReagentError> {
    match city.to_lowercase().trim() {
        "athens" => Ok(json!({
            "city": "Athens",
            "temperature_celsius": 28.5,
            "condition": "Sunny",
            "humidity": 42
        })),
        "cairo" => Ok(json!({
            "city": "Cairo",
            "temperature_celsius": 34.5,
            "condition": "Clear",
            "humidity": 28
        })),
        "london" => Ok(json!({
            "city": "London",
            "temperature_celsius": 16.0,
            "condition": "Overcast",
            "humidity": 75
        })),
        unknown => Err(ReagentError::recoverable(
            format!("No meteorological sensor data available for '{unknown}'"),
            "Supported cities in this station: Athens, Cairo, London",
        )),
    }
}

// ---------------------------------------------------------------------------
// 2. Main Execution
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("============================================================");
    println!(" ⚗ RUBEDO: Basic Artificer Reasoning Loop Example");
    println!("============================================================\n");

    // 1. Initialize Artificer with persona instructions and reagents
    let artificer = Artificer::builder()
        .name("HermesScholar")
        .instructions(
            "You are an analytical research assistant. When queried about comparative \
             measurements, look up data using your reagents and compute accurate averages.",
        )
        .model("claude-3-5-sonnet")
        .max_steps(5)
        .reagent(Arc::new(CalculateReagent))
        .reagent(Arc::new(FetchWeatherReagent))
        .build()?;

    println!("1. Artificer Initialized:");
    println!("   - Name:         {}", artificer.name);
    println!("   - Model:        {}", artificer.model);
    println!("   - Max Steps:    {}", artificer.max_steps);
    println!("   - Capabilities: {:?}\n", artificer.reagent_names());

    // 2. Inspect generated JSON Schemas
    println!("2. Generated Reagent JSON Schemas:");
    for schema in artificer.reagent_schemas() {
        let name = schema.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let desc = schema
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        println!("   - [{name}]: {desc}");
    }
    println!();

    // 3. User Query Simulation
    let query = "What is the average temperature between Athens and Cairo?";
    println!("3. Incoming User Query:\n   \"{query}\"\n");

    // 4. Simulate ReAct Execution Steps
    println!("4. Executing ReAct Reasoning Loop:");

    // Step 1: Fetch Athens weather
    let weather_tool = artificer
        .get_reagent("fetch_weather")
        .ok_or("Reagent 'fetch_weather' missing")?;

    println!("   [Step 1] Reasoning: Need temperature data for Athens.");
    let athens_data = weather_tool
        .transmute(json!({ "city": "Athens" }))
        .await?;
    let temp_athens = athens_data
        .get("temperature_celsius")
        .and_then(|v| v.as_f64())
        .ok_or("Missing temperature for Athens")?;
    println!("            Action Output: Athens = {temp_athens}°C");

    // Step 2: Fetch Cairo weather
    println!("   [Step 2] Reasoning: Need temperature data for Cairo.");
    let cairo_data = weather_tool
        .transmute(json!({ "city": "Cairo" }))
        .await?;
    let temp_cairo = cairo_data
        .get("temperature_celsius")
        .and_then(|v| v.as_f64())
        .ok_or("Missing temperature for Cairo")?;
    println!("            Action Output: Cairo = {temp_cairo}°C");

    // Step 3: Compute sum using calculator
    let calc_tool = artificer
        .get_reagent("calculator")
        .ok_or("Reagent 'calculator' missing")?;

    println!("   [Step 3] Reasoning: Sum temperatures ({temp_athens} + {temp_cairo}).");
    let sum_val = calc_tool
        .transmute(json!({
            "operation": "add",
            "a": temp_athens,
            "b": temp_cairo
        }))
        .await?;
    let sum = sum_val.as_f64().ok_or("Invalid calculation output")?;
    println!("            Action Output: Sum = {sum}");

    // Step 4: Compute average (sum / 2.0)
    println!("   [Step 4] Reasoning: Divide sum by 2 to determine average.");
    let avg_val = calc_tool
        .transmute(json!({
            "operation": "divide",
            "a": sum,
            "b": 2.0
        }))
        .await?;
    let average = avg_val.as_f64().ok_or("Invalid calculation output")?;
    println!("            Action Output: Average = {average}°C\n");

    // 5. Synthesize Verified Final Answer
    let final_answer = format!(
        "The current temperature in Athens is {temp_athens}°C and in Cairo is {temp_cairo}°C. \
         The verified average temperature between the two cities is {average}°C."
    );

    println!("5. Final Synthesized Response:");
    println!("   \"{final_answer}\"\n");
    println!("============================================================");
    println!(" Execution completed successfully with zero runtime errors.");
    println!("============================================================");

    Ok(())
}
