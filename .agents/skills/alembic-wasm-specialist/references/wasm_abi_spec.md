# Rubedo Alembic WASM ABI Specification

## 1. Packed Pointer & Length Return Convention

Guest execution functions return a single `u64` representing both the memory offset and the byte length of the output buffer:

$$\text{packed} = (\text{pointer} \ll 32) \mid (\text{length} \ \& \ \text{0xFFFF\_FFFF})$$

### Host Decoding (Rust)
```rust
let out_ptr = (packed >> 32) as usize;
let out_len = (packed & 0xFFFF_FFFF) as usize;
```

### Safety Rules for Host Memory Reads
1. **Bounds Check**: Ensure `out_ptr + out_len <= memory.data_size(&store)`. If the guest returns an out-of-bounds pointer, return an explicit error without reading memory.
2. **Deallocation**: The host MUST call `deallocate(out_ptr, out_len)` immediately after copying the bytes into host memory.
3. **Encoding**: All inputs and outputs across the boundary MUST be valid UTF-8 JSON.

---

## 2. Default Configuration Presets

| Configuration Field | Default Value | Notes |
|:---|:---|:---|
| `max_memory_bytes` | $67,108,864$ (64 MB) | 1,024 WebAssembly pages (64 KB each) |
| `fuel_limit` | $100,000,000$ | Approximately 5–20ms of intense computation |
| `async_support` | `true` | Allows async host function calls |
| `epoch_interruption` | `false` | Fuel metering preferred for deterministic reproducibility |
