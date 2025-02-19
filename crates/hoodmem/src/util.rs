/// Reinterpret the memory at `t_ptr` as something else
pub fn read_from_buffer<T: Copy>(buffer: &[u8], offset: usize) -> T {
    assert!(
        offset + size_of::<T>() <= buffer.len(),
        "Out of bounds read"
    );

    let ptr = buffer[offset..].as_ptr() as *const T;

    // SAFETY: The buffer is large enough to contain T
    unsafe { std::ptr::read_unaligned(ptr) }
}
