pub struct DraftSurface {
    surface: wgpu::Surface<'static>,
}

impl DraftSurface {
    pub fn new(surface: wgpu::Surface<'static>) -> Self {
        Self { surface }
    }
}
