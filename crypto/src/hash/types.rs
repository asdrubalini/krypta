/// Represents a Sha256 hash
#[derive(Default)]
pub struct Sha256Hash {
    hash: [u8; 32],
}

impl From<&[u8]> for Sha256Hash {
    /// Sha256 hash from bytes
    fn from(slice: &[u8]) -> Self {
        let mut hash = Self::default();
        hash.hash.copy_from_slice(slice);
        hash
    }
}

impl Sha256Hash {
    /// Convert self as an hex string
    pub fn as_hex(&self) -> String {
        self.hash.iter().map(|n| format!("{:02x}", n)).collect()
    }
}
