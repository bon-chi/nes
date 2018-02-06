struct CpuMemoryMap {
    map: [u8; 2 ^ 16],
}

impl CpuMemoryMap {
    fn fetch8(&self) -> u8 {
        return b'0';
    }
}
