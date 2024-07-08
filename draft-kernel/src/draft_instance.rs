use crate::draft_model::{DraftModel, DrawModel};

pub struct DraftInstance {
    pub model: DraftModel,
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub buffer: wgpu::Buffer,
}

impl DraftInstance {
    pub fn to_raw(position: glam::Vec3, rotation: glam::Quat) -> DraftInstanceRaw {
        DraftInstanceRaw {
            model: (glam::Mat4::from_translation(position) * glam::Mat4::from_quat(rotation))
                .to_cols_array_2d(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DraftInstanceRaw {
    model: [[f32; 4]; 4],
}

impl DraftInstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<DraftInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub trait DrawInstance<'a> {
    fn draw_instance(
        &mut self,
        instance: &'a DraftInstance,
        camera_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawInstance<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_instance(
        &mut self,
        instance: &'b DraftInstance,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(1, instance.buffer.slice(..));
        self.draw_model(&instance.model, camera_bind_group);
    }
}
