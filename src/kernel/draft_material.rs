use super::draft_texture::DraftTexture;

pub struct DraftMaterial {
    pub _name: String,
    pub _diffuse_texture: DraftTexture,
    pub bind_group: wgpu::BindGroup,
}
