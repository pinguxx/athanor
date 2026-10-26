//! # Example: Sandboxed Alembic WASM Execution & Fuel Metering
//!
//! Demonstrates running untrusted model-generated tools inside an isolated WebAssembly
//! sandbox using [`rubedo_alembic::AlembicSandbox`].
//!
//! Illustrates:
//! 1. In-process WASM sandbox instantiation with zero host filesystem/network capabilities.
//! 2. Bidirectional JSON exchange over guest linear memory via [`rubedo_alembic::WasmReagent`].
//! 3. Deterministic fuel metering halting an infinite loop in $<15$ milliseconds without
//!    crashing the host application process.

use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use rubedo_alembic::config::AlembicConfig;
use rubedo_alembic::error::AlembicError;
use rubedo_alembic::reagent::WasmReagent;
use rubedo_alembic::sandbox::AlembicSandbox;
use rubedo_core::reagent::Reagent;
use serde_json::json;

/// A WebAssembly guest module that acts as an echo/transmutation calculator over linear memory.
const GUEST_TRANSMUTATION_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (func (export "allocate") (param $size i32) (result i32)
    (i32.const 0)
  )
  (func (export "transmute") (param $ptr i32) (param $len i32) (result i64)
    (i64.or
      (i64.shl (i64.extend_i32_u (local.get $ptr)) (i64.const 32))
      (i64.extend_i32_u (local.get $len))
    )
  )
)
"#;

/// A malicious or buggy WebAssembly guest module containing an infinite instruction loop.
const INFINITE_LOOP_WAT: &str = r#"
(module
  (func (export "run")
    (loop (br 0))
  )
)
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("============================================================");
    println!(" ⚗ RUBEDO: Sandboxed Alembic WASM & Fuel Limits Example");
    println!("============================================================\n");

    // 1. Initialize Alembic sandbox with memory and fuel limits
    println!("1. Configuring Alembic WebAssembly Sandbox:");
    let memory_limit_mb = 16;
    let fuel_limit = 100_000; // 100k instructions

    let config = AlembicConfig::new()
        .with_max_memory_bytes(memory_limit_mb * 1024 * 1024)
        .with_fuel_limit(fuel_limit);

    let sandbox = Arc::new(AlembicSandbox::new(config)?);
    println!("   - Memory Ceiling: {memory_limit_mb} MB");
    println!("   - Fuel Budget:    {fuel_limit} instructions");
    println!("   - Host Access:    ZERO (no FS, no network, no env)\n");

    // 2. Part 1: Sandboxed computational capability execution
    println!("2. Part 1: Executing Safe Guest Module via WasmReagent:");
    let wasm_reagent = WasmReagent::new(
        "wasm_echo_transmuter",
        "Linear memory sandboxed echo transmuter",
        json!({
            "type": "object",
            "properties": {
                "element": { "type": "string" },
                "target": { "type": "string" }
            }
        }),
        GUEST_TRANSMUTATION_WAT.as_bytes(),
        sandbox.clone(),
    );

    let input_payload = json!({
        "element": "Aurum",
        "target": "PhilosopherStone",
        "cycle": 7
    });

    println!("   Input JSON Payload:  {}", input_payload);
    let output_payload = wasm_reagent.transmute(input_payload.clone()).await?;
    println!("   Output JSON Payload: {}\n", output_payload);

    // 3. Part 2: Malicious / Runaway Code Containment Demo
    println!("3. Part 2: Malicious / Runaway Code Containment:");
    println!("   Executing guest function containing an infinite loop: `(loop (br 0))`...");

    let start_time = Instant::now();
    let result = sandbox.execute_fn(INFINITE_LOOP_WAT.as_bytes(), "run");
    let elapsed = start_time.elapsed();

    match result {
        Err(AlembicError::FuelExhausted) => {
            println!("   [CONTAINED] Infinite loop trapped by Alembic fuel watchdog!");
            println!("   Abort Latency: {} ms (SLA is < 15 ms)", elapsed.as_millis());
            println!("   Host Process Status: Completely unaffected and running healthy.\n");

            if elapsed.as_millis() >= 15 {
                return Err(format!(
                    "Fuel abort exceeded 15ms SLA (took {} ms)",
                    elapsed.as_millis()
                )
                .into());
            }
        }
        other => {
            return Err(format!("Expected AlembicError::FuelExhausted, got: {other:?}").into());
        }
    }

    println!("============================================================");
    println!(" Sandboxed execution completed successfully with zero panics.");
    println!("============================================================");

    Ok(())
}
