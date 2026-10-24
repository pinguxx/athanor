use std::sync::Arc;
use rubedo_alembic::prelude::*;
use rubedo_core::artificer::Artificer;
use rubedo_core::reagent::{Reagent, ReagentError};
use serde_json::json;

const ECHO_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (func (export "allocate") (param $size i32) (result i32)
    (i32.const 0)
  )
  (func (export "deallocate") (param $ptr i32) (param $size i32)
    ;; No-op for test
  )
  (func (export "transmute") (param $ptr i32) (param $len i32) (result i64)
    ;; Return packed (ptr << 32) | len
    (i64.or
      (i64.shl (i64.extend_i32_u (local.get $ptr)) (i64.const 32))
      (i64.extend_i32_u (local.get $len))
    )
  )
)
"#;

const MEMORY_GROWTH_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (func (export "run")
    ;; Attempt to grow memory by 2000 pages (128 MB), exceeding 64 MB limit
    (drop (memory.grow (i32.const 2000)))
  )
)
"#;

#[test]
fn test_echo_linear_memory_roundtrip() {
    let sandbox = AlembicSandbox::new(AlembicConfig::default()).expect("sandbox");

    let input = json!({
        "agent": "rubedo",
        "transmutation": "alchemical",
        "count": 42
    });

    let output = sandbox
        .execute(ECHO_WAT.as_bytes(), input.clone())
        .expect("echo execution");

    assert_eq!(output, input);
}

#[tokio::test]
async fn test_wasm_reagent_integration_with_artificer() {
    let sandbox = Arc::new(AlembicSandbox::new(AlembicConfig::default()).expect("sandbox"));

    let reagent = Arc::new(WasmReagent::new(
        "wasm_echo",
        "Echoes JSON payloads through linear memory",
        json!({
            "type": "object",
            "properties": {
                "message": { "type": "string" }
            }
        }),
        ECHO_WAT.as_bytes(),
        sandbox.clone(),
    ));

    assert_eq!(reagent.name(), "wasm_echo");
    assert_eq!(reagent.description(), "Echoes JSON payloads through linear memory");
    assert_eq!(reagent.wasm_binary(), ECHO_WAT.as_bytes());

    let input = json!({ "message": "solve et coagula" });
    let output = reagent.transmute(input.clone()).await.expect("transmute");
    assert_eq!(output, input);

    // Register into Artificer
    let artificer = Artificer::builder()
        .name("Hermes")
        .model("claude-3-5-sonnet")
        .reagent(reagent)
        .build()
        .expect("artificer build");

    assert_eq!(artificer.reagent_names().len(), 1);
    let discovered = artificer.get_reagent("wasm_echo");
    assert!(discovered.is_some());
}

#[test]
fn test_memory_growth_limit_refusal() {
    let sandbox = AlembicSandbox::new(AlembicConfig::default()).expect("sandbox");

    // The module attempts to grow memory past 64MB limit; limiter safely refuses
    let res = sandbox.execute_fn(MEMORY_GROWTH_WAT.as_bytes(), "run");
    assert!(res.is_ok(), "Memory growth refusal should not crash the guest");
}

#[tokio::test]
async fn test_wasm_reagent_fuel_exhaustion_maps_to_fatal() {
    let config = AlembicConfig::new().with_fuel_limit(10_000);
    let sandbox = Arc::new(AlembicSandbox::new(config).expect("sandbox"));

    let loop_wat = r#"
    (module
      (func (export "run")
        (loop (br 0))
      )
    )
    "#;

    let reagent = WasmReagent::new(
        "infinite_loop",
        "Spins indefinitely",
        json!({}),
        loop_wat.as_bytes(),
        sandbox,
    );

    let res = reagent.transmute(json!({})).await;
    assert!(res.is_err());
    match res.unwrap_err() {
        ReagentError::Fatal(msg) => {
            assert!(msg.contains("WASM fuel exhausted"));
        }
        other => panic!("Expected ReagentError::Fatal, got {other:?}"),
    }
}

#[test]
fn test_memory_limit_exceeded_on_oversized_payload() {
    let config = AlembicConfig::new().with_max_memory_bytes(100);
    let sandbox = AlembicSandbox::new(config).expect("sandbox");

    let large_input = json!({ "data": "a".repeat(200) });
    let res = sandbox.execute(ECHO_WAT.as_bytes(), large_input);

    assert!(matches!(res, Err(AlembicError::MemoryLimitExceeded { requested: _, max: 100 })));
}

#[test]
fn test_missing_exports_detection() {
    let sandbox = AlembicSandbox::new(AlembicConfig::default()).expect("sandbox");

    let no_mem_wat = r#"
    (module
      (func (export "run"))
    )
    "#;

    let res = sandbox.execute(no_mem_wat.as_bytes(), json!({}));
    assert!(matches!(res, Err(AlembicError::MissingExport(_))));

    let run_res = sandbox.execute_fn(no_mem_wat.as_bytes(), "nonexistent");
    assert!(matches!(run_res, Err(AlembicError::MissingExport(_))));
}

#[test]
fn test_alembic_config_and_error_display() {
    let cfg = AlembicConfig::new()
        .with_max_memory_bytes(32 * 1024 * 1024)
        .with_fuel_limit(50_000_000)
        .with_wasi(false);

    assert_eq!(cfg.max_memory_bytes, 32 * 1024 * 1024);
    assert_eq!(cfg.fuel_limit, 50_000_000);
    assert!(!cfg.wasi_enabled);

    let err1 = AlembicError::FuelExhausted;
    assert_eq!(format!("{err1}"), "Fuel exhausted: execution exceeded instruction limit");

    let err2 = AlembicError::MemoryLimitExceeded { requested: 1000, max: 500 };
    assert_eq!(format!("{err2}"), "Memory limit exceeded: requested 1000 bytes, max 500 bytes");

    let err3 = AlembicError::Instantiation("failed link".into());
    assert_eq!(format!("{err3}"), "Guest instantiation error: failed link");

    let err4 = AlembicError::Trap("unreachable".into());
    assert_eq!(format!("{err4}"), "WASM execution trapped: unreachable");

    let err5 = AlembicError::OutOfBounds { ptr: 10, len: 20, size: 15 };
    assert_eq!(format!("{err5}"), "Memory access out of bounds: pointer 10, length 20, memory size 15");

    let err6 = AlembicError::MissingExport("missing_fn".into());
    assert_eq!(format!("{err6}"), "Missing exported symbol: missing_fn");
}

#[test]
fn test_alembic_error_to_reagent_error_conversions() {
    let e_fuel = ReagentError::from(AlembicError::FuelExhausted);
    assert!(matches!(e_fuel, ReagentError::Fatal(_)));

    let e_mem = ReagentError::from(AlembicError::MemoryLimitExceeded { requested: 10, max: 5 });
    assert!(matches!(e_mem, ReagentError::Fatal(_)));

    let e_trap = ReagentError::from(AlembicError::Trap("div by zero".into()));
    assert!(matches!(e_trap, ReagentError::Fatal(_)));

    let e_inst = ReagentError::from(AlembicError::Instantiation("link err".into()));
    assert!(matches!(e_inst, ReagentError::Fatal(_)));

    let e_export = ReagentError::from(AlembicError::MissingExport("run".into()));
    assert!(matches!(e_export, ReagentError::Fatal(_)));

    let e_bounds = ReagentError::from(AlembicError::OutOfBounds { ptr: 0, len: 10, size: 5 });
    assert!(matches!(e_bounds, ReagentError::Fatal(_)));

    let serde_err: Result<serde_json::Value, _> = serde_json::from_str("{ invalid json");
    let e_serde = ReagentError::from(AlembicError::Serialization(serde_err.unwrap_err()));
    assert!(matches!(e_serde, ReagentError::InvalidInput(_)));

    let utf8_err = String::from_utf8(vec![0xFF, 0xFE]);
    let e_utf8 = ReagentError::from(AlembicError::Utf8(utf8_err.unwrap_err()));
    assert!(matches!(e_utf8, ReagentError::Fatal(_)));
}

#[test]
fn test_pointer_packing_and_memory_helpers() {
    use rubedo_alembic::memory::{pack_ptr_len, unpack_ptr_len, read_guest_bytes, write_guest_bytes};
    use wasmtime::{Engine, Memory, MemoryType, Store};

    let packed = pack_ptr_len(1234, 5678);
    let (ptr, len) = unpack_ptr_len(packed);
    assert_eq!(ptr, 1234);
    assert_eq!(len, 5678);

    // Test memory read/write bounds checks
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let mem_type = MemoryType::new(1, Some(1));
    let memory = Memory::new(&mut store, mem_type).expect("memory");

    // Valid write & read
    let data = b"alembic_test_data";
    write_guest_bytes(&memory, &mut store, 0, data).expect("write");
    let read_back = read_guest_bytes(&memory, &store, 0, data.len()).expect("read");
    assert_eq!(read_back, data);

    // Out of bounds write
    let oob_write = write_guest_bytes(&memory, &mut store, 65530, data);
    assert!(matches!(oob_write, Err(AlembicError::OutOfBounds { .. })));

    // Out of bounds read
    let oob_read = read_guest_bytes(&memory, &store, 65530, 20);
    assert!(matches!(oob_read, Err(AlembicError::OutOfBounds { .. })));
}

#[test]
fn test_wasm_reagent_builder_accessors_and_debug() {
    let sandbox = Arc::new(AlembicSandbox::new(AlembicConfig::default()).expect("sandbox"));
    assert_eq!(sandbox.config().fuel_limit, 100_000_000);
    let _engine = sandbox.engine();

    let reagent = WasmReagent::new(
        "test_agent",
        "description test",
        json!({ "type": "number" }),
        ECHO_WAT.as_bytes(),
        sandbox,
    ).with_output_schema(json!({ "type": "string" }));

    assert_eq!(reagent.input_schema(), json!({ "type": "number" }));
    assert_eq!(reagent.output_schema(), json!({ "type": "string" }));

    let debug_str = format!("{reagent:?}");
    assert!(debug_str.contains("test_agent"));
    assert!(debug_str.contains("description test"));
}
