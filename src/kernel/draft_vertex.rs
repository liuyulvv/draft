pub trait DraftVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
