use super::{draft_material::DraftMaterial, draft_mesh::DraftMesh};

pub struct DraftModel {
    pub meshes: Vec<DraftMesh>,
    pub materials: Vec<DraftMaterial>,
}

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a DraftMesh,
        material: &'a DraftMaterial,
        camera_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_model(&mut self, model: &'a DraftModel, camera_bind_group: &'a wgpu::BindGroup);
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b DraftMesh,
        material: &'a DraftMaterial,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    fn draw_model(&mut self, model: &'b DraftModel, camera_bind_group: &'b wgpu::BindGroup) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh(mesh, material, camera_bind_group);
        }
    }
}
