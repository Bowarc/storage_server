#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct Size {
    original: u64,
    compressed: u64,
}

impl Size {
    pub fn new(original: u64, compressed: u64) -> Self {
        Self {
            original,
            compressed,
        }
    }
    pub fn original(&self) -> u64{
        self.original
    }
    pub fn compressed(&self) -> u64{
        self.compressed
    }
}
