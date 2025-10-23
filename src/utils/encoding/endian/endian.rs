pub fn write_u64_le(value: u64, buf: &mut [u8]) {
    buf[..8].copy_from_slice(&value.to_le_bytes());
}

pub fn write_u16_le(value: u16, buf: &mut [u8]) {
    buf[..2].copy_from_slice(&value.to_le_bytes());
}

pub fn write_u8(value: u8, buf: &mut [u8]) {
    buf[0] = value;
}

pub fn write_bytes(bytes: &[u8], buf: &mut [u8]) {
    buf[..bytes.len()].copy_from_slice(bytes);
}

/// -------------------- Read from slice --------------------
pub fn read_u64_le(buf: &[u8], offset: usize) -> Option<u64> {
    buf.get(offset..offset + 8)
        .map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap()))
}

pub fn read_u16_le(buf: &[u8], offset: usize) -> Option<u16> {
    buf.get(offset..offset + 2)
        .map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
}

pub fn read_u8(buf: &[u8], offset: usize) -> Option<u8> {
    buf.get(offset).copied()
}

pub fn read_bytes(buf: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    buf.get(offset..offset + len)
}
