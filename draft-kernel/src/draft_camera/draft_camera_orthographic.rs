use super::{
    draft_camera::{DraftCamera, DraftCameraType},
    draft_camera_projection::DraftCameraProjection,
};

pub struct DraftCameraOrthographic {
    position: glam::Vec3,
    target: glam::Vec3,
    projection: DraftCameraProjection,
}

impl DraftCameraOrthographic {
    pub fn new(position: glam::Vec3, target: glam::Vec3) -> Self {
        Self {
            position,
            target,
            projection: DraftCameraProjection::new(800, 600, 45.0, 0.1, 100.0),
        }
    }
}

impl DraftCamera for DraftCameraOrthographic {
    fn position(&self) -> glam::Vec3 {
        self.position
    }

    fn _camera_type(&self) -> DraftCameraType {
        DraftCameraType::Orthographic
    }

    fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.position, self.target, glam::Vec3::Y)
    }

    fn update_projection(&mut self, width: u32, height: u32) {
        self.projection.resize(width, height);
    }

    fn projection_matrix(&self) -> glam::Mat4 {
        self.projection.calc_matrix()
    }

    fn process_keyboard(
        &mut self,
        _physical_key: &winit::keyboard::PhysicalKey,
        _logical_key: &winit::keyboard::Key,
        _state: &winit::event::ElementState,
    ) -> bool {
        false
    }

    fn process_scroll(&mut self, _delta: &winit::event::MouseScrollDelta) -> bool {
        false
    }

    fn update_camera(&mut self, _dt: web_time::Duration) -> bool {
        false
    }
}
