use crate::{
    draft_camera::{
        draft_camera_manager::DraftCameraManager, draft_camera_uniform::DraftCameraUniform,
    },
    draft_instance::DraftInstanceRaw,
    draft_model::{DraftModelManager, DrawModel},
    draft_vertex::{DraftModelVertex, DraftVertexTrait},
};
use std::{borrow::BorrowMut, rc::Rc, sync::Arc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::{include_wgsl, util::DeviceExt};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;
use winit::{
    application::ApplicationHandler,
    event::{MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftAppType {
    #[cfg(not(target_arch = "wasm32"))]
    Desktop,
    #[cfg(target_arch = "wasm32")]
    Web,
}

pub struct Draft {
    instance: wgpu::Instance,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    surface_configuration: Option<wgpu::SurfaceConfiguration>,
    device: Option<Rc<wgpu::Device>>,
    queue: Option<Rc<wgpu::Queue>>,
    shader_module: Option<wgpu::ShaderModule>,

    triangle_pipeline: Option<wgpu::RenderPipeline>,

    width: u32,
    height: u32,

    texture_bind_group_layout: Option<Rc<wgpu::BindGroupLayout>>,

    camera_uniform: DraftCameraUniform,
    camera_buffer: Option<wgpu::Buffer>,
    camera_bind_group_layout: Option<wgpu::BindGroupLayout>,
    camera_bind_group: Option<wgpu::BindGroup>,

    camera_manager: DraftCameraManager,
    model_manager: Option<DraftModelManager>,

    mouse_pressed: bool,
    last_render_time: web_time::Instant,
}

impl Draft {
    pub fn new(app_type: DraftAppType) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let width = 600;
        let height = 400;
        let mut camera_manager = DraftCameraManager::new();
        let mut camera_uniform = DraftCameraUniform::new();
        camera_manager.update_view_proj(&mut camera_uniform);
        match app_type {
            #[cfg(not(target_arch = "wasm32"))]
            DraftAppType::Desktop => Self {
                instance,
                window: None,
                surface: None,
                surface_configuration: None,
                device: None,
                queue: None,
                shader_module: None,
                triangle_pipeline: None,
                width,
                height,
                texture_bind_group_layout: None,
                camera_uniform,
                camera_buffer: None,
                camera_bind_group_layout: None,
                camera_bind_group: None,
                camera_manager,
                model_manager: None,
                mouse_pressed: false,
                last_render_time: web_time::Instant::now(),
            },
            #[cfg(target_arch = "wasm32")]
            DraftAppType::Web => {
                let window = web_sys::window().expect("No global `window` exists");
                let document = window.document().expect("Should have a document on window");
                let canvas = document.get_element_by_id("main_canvas").unwrap();
                let canvas = canvas
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .expect("Show have a canvas");
                let surface = Draft::create_surface(&instance, canvas);
                Self {
                    instance,
                    window: None,
                    surface: Some(surface),
                    surface_configuration: None,
                    device: None,
                    queue: None,
                    shader_module: None,
                    triangle_pipeline: None,
                    width,
                    height,
                    texture_bind_group_layout: None,
                    camera_uniform,
                    camera_buffer: None,
                    camera_bind_group_layout: None,
                    camera_bind_group: None,
                    camera_manager,
                    model_manager: None,
                    mouse_pressed: false,
                    last_render_time: web_time::Instant::now(),
                }
            }
        }
    }

    pub async fn init(&mut self, width: u32, height: u32) {
        let adapter = self
            .instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: self.surface.as_ref(),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        self.device = Some(Rc::new(device));
        self.queue = Some(Rc::new(queue));
        let caps = self.surface.as_ref().unwrap().get_capabilities(&adapter);
        self.surface_configuration = Some(wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        });
        self.surface.as_ref().unwrap().configure(
            self.device.as_ref().unwrap(),
            self.surface_configuration.as_ref().unwrap(),
        );

        self.texture_bind_group_layout = Some(Rc::new(
            self.device
                .clone()
                .unwrap()
                .borrow_mut()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                }),
        ));

        self.model_manager = Some(DraftModelManager::new(
            self.device.clone().unwrap(),
            self.queue.clone().unwrap(),
            self.texture_bind_group_layout.clone().unwrap(),
        ));

        self.camera_buffer = Some(self.device.as_ref().unwrap().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[self.camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));
        self.camera_bind_group_layout =
            Some(self.device.as_ref().unwrap().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                },
            ));
        self.camera_bind_group = Some(self.device.as_ref().unwrap().create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: self.camera_bind_group_layout.as_ref().unwrap(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_ref().unwrap().as_entire_binding(),
                }],
            },
        ));

        self.shader_module = Some(
            self.device
                .as_ref()
                .unwrap()
                .create_shader_module(include_wgsl!("shader/shader.wgsl")),
        );
        self.create_triangle_pipeline();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.as_ref().unwrap().get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.device
                .as_ref()
                .unwrap()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(self.triangle_pipeline.as_ref().unwrap());
            let manager_models = self.model_manager.as_ref().unwrap().models();
            let models = manager_models.0;
            let instances = manager_models.1;
            let instances_buffer = manager_models.2;
            for ((model, instance), instance_buffer) in models
                .iter()
                .zip(instances.iter())
                .zip(instances_buffer.iter())
            {
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.draw_model_instanced(
                    model,
                    instance,
                    self.camera_bind_group.as_ref().unwrap(),
                );
            }
        }
        self.queue
            .as_ref()
            .unwrap()
            .submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn create_surface(&mut self, window: Arc<Window>) {
        let surface = self.instance.create_surface(window.clone()).unwrap();
        self.surface = Some(surface);
    }

    #[cfg(target_arch = "wasm32")]
    fn create_surface(
        instance: &wgpu::Instance,
        window: web_sys::HtmlCanvasElement,
    ) -> wgpu::Surface<'static> {
        let surface_target = wgpu::SurfaceTarget::Canvas(window);
        let surface = instance.create_surface(surface_target).unwrap();
        surface
    }

    fn create_triangle_pipeline(&mut self) {
        self.triangle_pipeline =
            Some(self.create_render_pipeline(wgpu::PrimitiveTopology::TriangleList));
    }

    fn create_render_pipeline(
        &mut self,
        primitive: wgpu::PrimitiveTopology,
    ) -> wgpu::RenderPipeline {
        let layout =
            self.device
                .as_ref()
                .unwrap()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        self.texture_bind_group_layout.as_ref().unwrap(),
                        self.camera_bind_group_layout.as_ref().unwrap(),
                    ],
                    push_constant_ranges: &[],
                });
        self.device
            .as_ref()
            .unwrap()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: self.shader_module.as_ref().unwrap(),
                    compilation_options: Default::default(),
                    entry_point: "vs_main",
                    buffers: &[DraftModelVertex::desc(), DraftInstanceRaw::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: self.shader_module.as_ref().unwrap(),
                    compilation_options: Default::default(),
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.surface_configuration.as_ref().unwrap().format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: primitive,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.camera_manager.resize_camera(width, height);
        self.surface_configuration.as_mut().unwrap().width = width;
        self.surface_configuration.as_mut().unwrap().height = height;
        self.surface.as_mut().unwrap().configure(
            self.device.as_ref().unwrap(),
            self.surface_configuration.as_ref().unwrap(),
        );
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => self.camera_manager.active_camera().process_keyboard(
                &event.physical_key,
                &event.logical_key,
                &event.state,
            ),
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                self.camera_manager.active_camera().process_scroll(&delta);
                true
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button: MouseButton::Left,
            } => {
                self.mouse_pressed = state.is_pressed();
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: web_time::Duration) {
        self.camera_manager.active_camera().update_camera(dt);
        self.camera_manager
            .update_view_proj(&mut self.camera_uniform);
        self.queue.as_ref().unwrap().write_buffer(
            self.camera_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}

impl ApplicationHandler for Draft {
    #[cfg(not(target_arch = "wasm32"))]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("Draft");
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("Create window err."),
            );
            self.window = Some(window.clone());
            self.create_surface(window.clone());
            pollster::block_on(self.init(600, 400));
            self.last_render_time = web_time::Instant::now();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = web_sys::window().expect("No global `window` exists");
            let document = window.document().expect("Should have a document on window");
            let canvas = document.get_element_by_id("main_canvas").unwrap();
            let canvas = canvas
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("Show have a canvas");
            let win_attr = Window::default_attributes()
                .with_title("Draft")
                .with_canvas(Some(canvas))
                .with_prevent_default(false);
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("Create window err."),
            );
            self.window = Some(window.clone());
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if !self.input(&event) {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        self.resize(size.width, size.height);
                    }
                }
                WindowEvent::KeyboardInput {
                    device_id: _,
                    event,
                    is_synthetic: _,
                } => match event.physical_key {
                    #[cfg(not(target_arch = "wasm32"))]
                    PhysicalKey::Code(KeyCode::Enter) => {
                        pollster::block_on(
                            self.model_manager.as_mut().unwrap().add_model("cube.obj"),
                        );
                    }
                    _ => {}
                },
                WindowEvent::RedrawRequested => {
                    let now = web_time::Instant::now();
                    let dt = now - self.last_render_time;
                    self.last_render_time = now;
                    self.update(dt);
                    match self.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => self.resize(self.width, self.height),
                        Err(e) => {
                            eprintln!("{:?}", e);
                        }
                    }
                    self.window.as_ref().unwrap().request_redraw();
                }
                _ => {}
            }
        }
    }
}
