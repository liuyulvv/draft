#[cfg(not(target_arch = "wasm32"))]
use winit::{
    event::ElementState,
    keyboard::{Key, KeyCode, PhysicalKey},
};

use super::{
    draft_camera::DraftCamera, draft_camera_orthographic::DraftCameraOrthographic,
    draft_camera_perspective::DraftCameraPerspective, draft_camera_uniform::DraftCameraUniform,
};

pub struct DraftCameraManager {
    camera: Vec<Box<dyn DraftCamera>>,
    active_camera: usize,
}

impl DraftCameraManager {
    pub fn new() -> Self {
        Self {
            camera: vec![
                Box::new(DraftCameraOrthographic::new(
                    glam::Vec3::new(0.0, 0.0, 10.0),
                    glam::Vec3::new(0.0, 0.0, 0.0),
                )),
                Box::new(DraftCameraPerspective::new(
                    glam::Vec3::new(0.0, 0.0, 10.0),
                    glam::Vec3::new(0.0, 0.0, 0.0),
                )),
            ],
            active_camera: 0,
        }
    }

    pub fn set_active_camera(&mut self, index: usize) {
        if index < self.camera.len() {
            self.active_camera = index;
        }
    }

    pub fn active_camera(&mut self) -> &mut dyn DraftCamera {
        &mut *self.camera[self.active_camera]
    }

    pub fn resize_camera(&mut self, width: u32, height: u32) {
        for camera in self.camera.iter_mut() {
            camera.update_projection(width, height);
        }
    }

    pub fn update_view_proj(&mut self, camera_uniform: &mut DraftCameraUniform) {
        let camera = self.active_camera();
        let camera_matrix = camera.projection_matrix() * camera.view_matrix();
        camera_uniform.update_view_proj(camera.position(), camera_matrix);
    }

    pub fn process_keyboard(
        &mut self,
        physical_key: &PhysicalKey,
        logical_key: &Key,
        state: &ElementState,
    ) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        match physical_key {
            PhysicalKey::Code(KeyCode::Numpad2) | PhysicalKey::Code(KeyCode::Digit2) => {
                self.set_active_camera(0)
            }
            PhysicalKey::Code(KeyCode::Numpad3) | PhysicalKey::Code(KeyCode::Digit3) => {
                self.set_active_camera(1)
            }
            _ => {}
        }
        self.active_camera()
            .process_keyboard(physical_key, logical_key, state)
    }
}
