use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};
mod transformer;

/// Maps the address of a shared buffer to a reference counted buffer.
///
/// Buffers are wrapped in a reference counted smart pointer to ensure that they are not freed while in use.
static SHARED_BUFFERS: LazyLock<Mutex<HashMap<usize, Arc<Box<[u8]>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Get a buffer that can be written to.
///
/// The buffer needs to be freed with `free_shared_buffer`.
///
/// # Safety
///
/// Will crash if the guest has no more memory available.
#[unsafe(no_mangle)]
pub extern "C" fn allocate_shared_buffer(size: usize) -> usize {
    // TODO: Convert to `new_zeroed_slice` once it's stabilized.
    let mut buffer = Box::<[u8]>::new_uninit_slice(size);
    for byte in buffer.iter_mut() {
        let _ = *byte.write(0);
    }
    // SAFETY: We just initialized the buffer.
    let buffer = unsafe { buffer.assume_init() };

    share_buffer(buffer)
}

/// Make a buffer available to the host. Returns the memory address of the buffer.
fn share_buffer(buffer: Box<[u8]>) -> usize {
    let buffer_in_arc = Arc::new(buffer);
    let address = buffer_in_arc.as_ptr() as usize;
    SHARED_BUFFERS
        .lock()
        .unwrap()
        .insert(address, buffer_in_arc);
    address
}

/// Free a buffer that was allocated with `allocate_shared_buffer`.
///
/// - Returns 0 if the buffer was successfully freed.
/// - Returns 1 if the buffer is not currently allocated.
///
/// If the guest is currently using the buffer, it will also return 0, but the buffer will be freed once the guest is done with it.
#[unsafe(no_mangle)]
pub extern "C" fn free_shared_buffer(pointer: usize) -> u8 {
    let buffer = SHARED_BUFFERS.lock().unwrap().remove(&pointer);
    match buffer {
        Some(_) => 0,
        None => 1,
    }
}

#[repr(C)]
pub struct ProcessingResult {
    /// Whether the operation was successfull.
    ///
    /// If false, the pointer points to an error message.
    /// If true, the pointer points to the result.
    success: bool,
    /// Pointer to the result or an error message.
    ///
    /// The host is responsible for freeing the buffer using `free_shared_buffer`.
    pointer: usize,
    /// Length of the result or error message in bytes.
    length: usize,
}

/// Process the input buffer and return a new buffer.
///
#[unsafe(no_mangle)]
pub extern "C" fn process(input_buffer: usize) -> ProcessingResult {
    let (success, output) = match process_to_result(input_buffer) {
        Ok(output) => (true, output),
        Err(error) => (false, error),
    };

    let boxed_bytes = output.into_boxed_str().into_boxed_bytes();
    let output_length = boxed_bytes.len();
    let output_address = share_buffer(boxed_bytes);
    return ProcessingResult {
        success,
        pointer: output_address,
        length: output_length,
    };
}

/// Decode the input buffer and return the result with String as the error type.
fn process_to_result(input_buffer: usize) -> Result<String, String> {
    let input = SHARED_BUFFERS
        .lock()
        .map_err(|e| e.to_string())?
        .get(&input_buffer)
        .ok_or(
            "The input buffer does not exist. Use `allocate_shared_buffer` to allocate a buffer.",
        )?
        .clone();

    let input = std::str::from_utf8(&input).map_err(|e| e.to_string())?;

    transformer::rainbow_text(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn how_the_host_would_use_this() {
        let input = "Hello, world!";
        let input_bytes = input.as_bytes();
        let shared_pointer = allocate_shared_buffer(input_bytes.len());
        let shared_buffer =
            unsafe { std::slice::from_raw_parts_mut(shared_pointer as *mut u8, input_bytes.len()) };
        shared_buffer.copy_from_slice(input_bytes);
        let result = process(shared_pointer);
        assert_eq!(result.success, true);
        let output =
            unsafe { std::slice::from_raw_parts(result.pointer as *const u8, result.length) };
        let output = std::str::from_utf8(output).unwrap();
        assert_eq!(
            output,
            "\x1b[31mH\x1b[33me\x1b[32ml\x1b[36ml\x1b[34mo\x1b[35m,\x1b[31m \x1b[33mw\x1b[32mo\x1b[36mr\x1b[34ml\x1b[35md\x1b[31m!\x1b[0m"
        );
    }
}
