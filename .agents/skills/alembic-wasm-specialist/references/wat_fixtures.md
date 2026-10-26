# Canonical WebAssembly Text (.wat) Test Fixtures

These fixtures provide minimal, zero-dependency WebAssembly modules used in `athanor-alembic` unit and integration tests.

---

## 1. Infinite Loop Fixture (`infinite_loop.wat`)
Used to verify that fuel metering terminates rogue execution in $<15$ milliseconds:

```wat
(module
  (func (export "run")
    (loop (br 0))
  )
  (func (export "transmute") (param i32 i32) (result i64)
    (loop (br 0))
    (i64.const 0)
  )
)
```

---

## 2. Minimal Echo Fixture (`echo.wat`)
Echoes the input pointer and length back directly to the host:

```wat
(module
  (memory (export "memory") 1)
  (func (export "allocate") (param $size i32) (result i32)
    (i32.const 0)
  )
  (func (export "deallocate") (param $ptr i32) (param $size i32)
    ;; No-op for test
  )
  (func (export "transmute") (param $ptr i32) (param $len i32) (result i64)
    ;; Return (ptr << 32) | len
    (i64.or
      (i64.shl (i64.extend_i32_u (local.get $ptr)) (i64.const 32))
      (i64.extend_i32_u (local.get $len))
    )
  )
)
```
