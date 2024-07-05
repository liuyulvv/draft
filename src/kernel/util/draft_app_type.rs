#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftAppType {
    #[cfg(not(target_arch = "wasm32"))]
    Desktop,
    #[cfg(target_arch = "wasm32")]
    Web,
}
