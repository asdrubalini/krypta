pub struct Sha256Hash<'a> {
    hash: &'a [u8; 32]
}

impl<'a> Sha256Hash<'a> {
    /// Convert self as an hex string
    pub fn as_hex(&self) -> String {
        self.hash.iter().map(|n| format!("{:02x}", n)).collect()
    }
}
