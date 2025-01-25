use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};
mod transformer;

/// A shared buffer that can be accessed by the host.
///
/// The guest holds a clone of the rc while using the buffer to make sure that the host does not free the buffer while the guest is still using it.
type SharedBuffer = Arc<Box<[u8]>>;
/// Keeps track of all shared buffers.
static SHARED_BUFFERS: LazyLock<Mutex<HashMap<usize, SharedBuffer>>> =
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

    share_buffer(buffer) as usize
}

/// Make a buffer available to the host. Returns the memory address of the buffer.
fn share_buffer(buffer: Box<[u8]>) -> *const u8 {
    let buffer_in_arc = Arc::new(buffer);
    let address = buffer_in_arc.as_ptr();
    SHARED_BUFFERS
        .lock()
        .unwrap()
        .insert(address as usize, buffer_in_arc);
    address
}

/// Free a buffer that was allocated with `allocate_shared_buffer`.
///
/// - Returns false if the buffer is not currently allocated or an error occurred.
/// - Returns true if the buffer was successfully freed.
///
/// If the guest is currently using the buffer, it will also return 0, but the buffer will be freed once the guest is done with it.
#[unsafe(no_mangle)]
pub extern "C" fn free_shared_buffer(pointer: usize) -> bool {
    let buffer = SHARED_BUFFERS.lock().unwrap().remove(&{ pointer });
    buffer.is_some()
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
    pointer: u64,
    /// Length of the result or error message in bytes.
    length: u64,
}

/// Process the input buffer and return a new buffer.
/// The new buffer needs to be freed with `free_shared_buffer`.
///
/// The first byte of the returned buffer is a boolean indicating whether the operation was successfull.
/// If false, the string contains an error message, otherwise it contains the result.
/// The next 4 bytes are the length of the string.
/// Then the string follows.
#[unsafe(no_mangle)]
pub extern "C" fn process(input_buffer: usize) -> usize {
    let (success, output) = match process_to_result(input_buffer) {
        Ok(output) => (true, output),
        Err(error) => (false, error),
    };

    let mut return_bytes = Vec::<u8>::with_capacity(output.len() + size_of::<usize>() + 1);
    return_bytes.push(success as u8);
    return_bytes.extend_from_slice(&output.len().to_le_bytes());
    return_bytes.extend_from_slice(output.as_bytes());

    share_buffer(return_bytes.into_boxed_slice()) as usize
}

/// Decode the input buffer and return the result with String as the error type.
fn process_to_result(input_buffer: usize) -> Result<String, String> {
    let input = SHARED_BUFFERS
        .lock()
        .map_err(|e| e.to_string())?
        .get(&{ input_buffer })
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

        let result = process(shared_pointer) as *const u8;
        let success = unsafe { *result } != 0;
        let length = unsafe { *(result.add(1) as *const [u8; size_of::<usize>()]) };
        let length = usize::from_le_bytes(length);
        let output = unsafe {
            std::slice::from_raw_parts(
                result.add(1 + size_of::<usize>()) as *const u8,
                length,
            )
        };
        assert!(success);

        let output = std::str::from_utf8(output).unwrap();
        assert_eq!(
            output,
            "\x1b[31mH\x1b[33me\x1b[32ml\x1b[36ml\x1b[34mo\x1b[35m,\x1b[31m \x1b[33mw\x1b[32mo\x1b[36mr\x1b[34ml\x1b[35md\x1b[31m!\x1b[0m"
        );
    }
}
