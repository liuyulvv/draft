#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DraftCameraType {
    Orthographic,
    Perspective,
}

pub trait DraftCamera {
    fn _camera_type(&self) -> DraftCameraType;

    fn position(&self) -> glam::Vec3;
    fn view_matrix(&self) -> glam::Mat4;

    fn update_projection(&mut self, width: u32, height: u32);
    fn projection_matrix(&self) -> glam::Mat4;

    fn process_keyboard(
        &mut self,
        physical_key: &winit::keyboard::PhysicalKey,
        logical_key: &winit::keyboard::Key,
        state: &winit::event::ElementState,
    ) -> bool;
    fn process_scroll(&mut self, delta: &winit::event::MouseScrollDelta) -> bool;
    fn update_camera(&mut self, dt: web_time::Duration) -> bool;
}
