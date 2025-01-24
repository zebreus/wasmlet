mod transformer;

// pub extern "C" fn process(input_ptr: *const u8, input_len: usize) -> u64 {
//     // SAFETY: The caller must provide a valid pointer and length.
//     let bytes = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };

//     // let string =

//     // 2. Transform the string somehow (reverse it, count words, etc.).
//     // 3. Allocate buffer for the output string and return pointer+length encoded in a single u64 or two separate exports.
//     // 4. If there's an error, consider how you’ll handle or signal it.
//     //    (One approach: return a null pointer or a special length, or store an error message in some known location.)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
