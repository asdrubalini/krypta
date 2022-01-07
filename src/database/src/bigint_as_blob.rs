use byteorder::{ByteOrder, LittleEndian};

pub struct BigIntAsBlob;

impl BigIntAsBlob {
    pub fn from_bytes(bytes: &[u8]) -> u64 {
        LittleEndian::read_u64(bytes)
    }

    pub fn from_u64(number: &u64) -> Vec<u8> {
        let mut buf = [0u8; 8];
        LittleEndian::write_u64(&mut buf, *number);
        Vec::from(buf)
    }
}
