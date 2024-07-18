#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DraftCameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl DraftCameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0, 0.0, 10.0, 1.0],
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera_position: glam::Vec3, camera_matrix: glam::Mat4) {
        self.view_position = camera_position.extend(1.0).into();
        self.view_proj = camera_matrix.to_cols_array_2d();
    }
}
