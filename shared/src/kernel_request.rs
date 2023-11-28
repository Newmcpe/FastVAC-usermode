#[repr(C)]
#[derive(Copy, Clone)]
pub struct KernelRequest {
    pub key: i32,
    pub operation: u8,
    pub process_id: u32,
    pub address: u64,
    pub size: usize,
}