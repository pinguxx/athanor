//! Host-guest linear memory bridge protocol for WebAssembly capability exchange.

use wasmtime::{Instance, Memory, Store, Trap};

use crate::error::AlembicError;

/// Pack a 32-bit pointer and 32-bit buffer length into a single 64-bit integer convention.
#[inline]
pub fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((ptr as u64) << 32) | ((len as u64) & 0xFFFF_FFFF)
}

/// Unpack a 64-bit packed ABI return value into a memory offset and buffer length.
#[inline]
pub fn unpack_ptr_len(packed: u64) -> (usize, usize) {
    let ptr = (packed >> 32) as usize;
    let len = (packed & 0xFFFF_FFFF) as usize;
    (ptr, len)
}

/// Write host bytes into guest linear memory at the specified offset with bounds verification.
pub fn write_guest_bytes<T>(
    memory: &Memory,
    store: &mut Store<T>,
    ptr: usize,
    bytes: &[u8],
) -> Result<(), AlembicError> {
    let size = memory.data_size(&*store);
    let end = ptr
        .checked_add(bytes.len())
        .ok_or(AlembicError::OutOfBounds {
            ptr,
            len: bytes.len(),
            size,
        })?;

    if end > size {
        return Err(AlembicError::OutOfBounds {
            ptr,
            len: bytes.len(),
            size,
        });
    }

    memory
        .write(store, ptr, bytes)
        .map_err(|e| AlembicError::Trap(e.to_string()))
}

/// Read bytes from guest linear memory from the specified offset with bounds verification.
pub fn read_guest_bytes<T>(
    memory: &Memory,
    store: &Store<T>,
    ptr: usize,
    len: usize,
) -> Result<Vec<u8>, AlembicError> {
    let size = memory.data_size(store);
    let end = ptr.checked_add(len).ok_or(AlembicError::OutOfBounds {
        ptr,
        len,
        size,
    })?;

    if end > size {
        return Err(AlembicError::OutOfBounds {
            ptr,
            len,
            size,
        });
    }

    let mut buffer = vec![0u8; len];
    memory
        .read(store, ptr, &mut buffer)
        .map_err(|e| AlembicError::Trap(e.to_string()))?;

    Ok(buffer)
}

/// Execute the 3-function linear memory exchange ABI (`allocate`, `transmute`, `deallocate`).
pub fn execute_exchange<T>(
    instance: &Instance,
    store: &mut Store<T>,
    memory: &Memory,
    input_bytes: &[u8],
) -> Result<Vec<u8>, AlembicError> {
    // 1. Allocate buffer in guest linear memory
    let in_ptr = if !input_bytes.is_empty() {
        let allocate = instance
            .get_typed_func::<u32, u32>(&mut *store, "allocate")
            .map_err(|_| AlembicError::MissingExport("allocate".into()))?;

        let guest_ptr = allocate
            .call(&mut *store, input_bytes.len() as u32)
            .map_err(map_trap)?;

        // Write input bytes into guest memory
        write_guest_bytes(memory, store, guest_ptr as usize, input_bytes)?;
        guest_ptr
    } else {
        0
    };

    // 2. Invoke capability entrypoint `transmute(ptr, len) -> u64`
    let transmute = instance
        .get_typed_func::<(u32, u32), u64>(&mut *store, "transmute")
        .map_err(|_| AlembicError::MissingExport("transmute".into()))?;

    let packed = transmute
        .call(&mut *store, (in_ptr, input_bytes.len() as u32))
        .map_err(map_trap)?;

    // 3. Unpack output pointer and length
    let (out_ptr, out_len) = unpack_ptr_len(packed);

    // 4. Read serialized output buffer from guest memory
    let output_bytes = read_guest_bytes(memory, store, out_ptr, out_len)?;

    // 5. Deallocate output buffer if module exports `deallocate`
    if let Ok(deallocate) = instance.get_typed_func::<(u32, u32), ()>(&mut *store, "deallocate") {
        let _ = deallocate.call(&mut *store, (out_ptr as u32, out_len as u32));
    }

    Ok(output_bytes)
}

/// Convert wasmtime error to strongly-typed AlembicError, mapping fuel exhaustion cleanly.
pub fn map_trap(err: wasmtime::Error) -> AlembicError {
    if let Some(trap) = err.downcast_ref::<Trap>() {
        if matches!(trap, Trap::OutOfFuel) {
            return AlembicError::FuelExhausted;
        }
        return AlembicError::Trap(trap.to_string());
    }
    AlembicError::Trap(err.to_string())
}
