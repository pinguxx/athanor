use std::time::Instant;
use rubedo_alembic::prelude::*;

const INFINITE_LOOP_WAT: &str = r#"
(module
  (func (export "run")
    (loop (br 0))
  )
)
"#;

const INFINITE_LOOP_TRANSMUTE_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (func (export "allocate") (param $size i32) (result i32)
    (i32.const 0)
  )
  (func (export "transmute") (param $ptr i32) (param $len i32) (result i64)
    (loop (br 0))
    (i64.const 0)
  )
)
"#;

#[test]
fn test_fuel_exhaustion_infinite_loop_halts_in_under_15ms() {
    let config = AlembicConfig::new()
        .with_fuel_limit(100_000);

    let sandbox = AlembicSandbox::new(config).expect("sandbox initialization");

    let start = Instant::now();
    let result = sandbox.execute_fn(INFINITE_LOOP_WAT.as_bytes(), "run");
    let elapsed = start.elapsed();

    // 1. Assert host execution returns Err(AlembicError::FuelExhausted)
    assert!(
        matches!(result, Err(AlembicError::FuelExhausted)),
        "Expected FuelExhausted, got: {result:?}"
    );

    // 2. Assert the infinite loop execution halts in < 15 milliseconds
    assert!(
        elapsed.as_millis() < 15,
        "Execution exceeded 15ms SLA: elapsed was {} ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_fuel_exhaustion_transmute_protocol() {
    let config = AlembicConfig::new()
        .with_fuel_limit(50_000);

    let sandbox = AlembicSandbox::new(config).expect("sandbox initialization");

    let start = Instant::now();
    let result = sandbox.execute(
        INFINITE_LOOP_TRANSMUTE_WAT.as_bytes(),
        serde_json::json!({ "input": "test" }),
    );
    let elapsed = start.elapsed();

    assert!(matches!(result, Err(AlembicError::FuelExhausted)));
    assert!(elapsed.as_millis() < 15);
}
