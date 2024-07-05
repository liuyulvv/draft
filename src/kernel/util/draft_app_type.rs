#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftAppType {
    Desktop,
    #[cfg(target_arch = "wasm32")]
    Web(String),
}
