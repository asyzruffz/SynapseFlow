/// Leaves a deliberate seam for the production transport introduced in milestone 3.
pub trait Transport: Send + Sync {
    fn is_available(&self) -> bool;
}
