/// Reinterpret the memory at `t_ptr` as something else
pub fn read_from_buffer<T: Copy>(buffer: &Vec<u8>, offset: u64) -> T {
    unsafe { *std::mem::transmute::<u64, *const T>(buffer.as_ptr() as u64 + offset) }
}
