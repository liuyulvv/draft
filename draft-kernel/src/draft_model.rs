use crate::{
    draft_instance::DraftInstance, draft_material::DraftMaterial, draft_mesh::DraftMesh, draft_util,
};
use std::{collections::HashMap, rc::Rc};
use wgpu::util::DeviceExt;

pub struct DraftModel {
    pub meshes: Vec<DraftMesh>,
    pub materials: Vec<DraftMaterial>,
}

pub trait DrawModel<'a> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a DraftMesh,
        material: &'a DraftMaterial,
        instances: &'a Vec<DraftInstance>,
        camera_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_model_instanced(
        &mut self,
        model: &'a DraftModel,
        instances: &'a Vec<DraftInstance>,
        camera_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b DraftMesh,
        material: &'b DraftMaterial,
        instances: &'b Vec<DraftInstance>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, 0..instances.len() as _);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b DraftModel,
        instances: &'b Vec<DraftInstance>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances, camera_bind_group);
        }
    }
}

pub struct DraftModelManager {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    texture_bind_group_layout: Rc<wgpu::BindGroupLayout>,
    models: HashMap<String, DraftModel>,
    instances: HashMap<String, Vec<DraftInstance>>,
    instances_buffer: HashMap<String, wgpu::Buffer>,
}

impl DraftModelManager {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        texture_bind_group_layout: Rc<wgpu::BindGroupLayout>,
    ) -> Self {
        Self {
            device,
            queue,
            texture_bind_group_layout,
            models: HashMap::new(),
            instances: HashMap::new(),
            instances_buffer: HashMap::new(),
        }
    }

    pub async fn add_model(&mut self, file_name: &str, position: Option<glam::Vec3>) {
        let exist = self.models.get(file_name);
        if exist.is_some() {
            let instance = DraftInstance {
                position: match position {
                    Some(p) => p,
                    None => glam::Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                rotation: glam::Quat::from_axis_angle(glam::Vec3::Z, 0.0),
            };
            self.instances.get_mut(file_name).unwrap().push(instance);
            let instances_data = self
                .instances
                .get(file_name)
                .unwrap()
                .iter()
                .map(DraftInstance::to_raw)
                .collect::<Vec<_>>();
            let instance_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Instance Buffer {:?}", file_name)),
                        contents: bytemuck::cast_slice(&instances_data),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            self.instances_buffer
                .insert(file_name.to_string(), instance_buffer);
        } else {
            let model = draft_util::load_model(
                file_name,
                self.device.clone(),
                self.queue.clone(),
                self.texture_bind_group_layout.clone(),
            )
            .await;
            match model {
                Ok(model) => {
                    self.models.insert(file_name.to_string(), model);
                    let instance = DraftInstance {
                        position: match position {
                            Some(p) => p,
                            None => glam::Vec3 {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            },
                        },
                        rotation: glam::Quat::from_axis_angle(glam::Vec3::Z, 0.0),
                    };
                    let instance_buffer =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&format!("Instance Buffer {:?}", file_name)),
                                contents: bytemuck::cast_slice(&[instance.to_raw()]),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                    self.instances.insert(file_name.to_string(), vec![instance]);
                    self.instances_buffer
                        .insert(file_name.to_string(), instance_buffer);
                }
                Err(e) => {
                    log::error!("Failed to load {}: {}", file_name, e);
                }
            }
        }
    }

    pub fn models(
        &self,
    ) -> (
        Vec<&DraftModel>,
        Vec<&Vec<DraftInstance>>,
        Vec<&wgpu::Buffer>,
    ) {
        let models: Vec<_> = self.models.values().collect();
        let instances: Vec<_> = self.instances.values().collect();
        let instances_buffer: Vec<_> = self.instances_buffer.values().collect();
        (models, instances, instances_buffer)
    }
}
