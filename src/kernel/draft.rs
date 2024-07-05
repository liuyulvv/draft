use super::{
    draft_model::DraftModelVertex, draft_vertex::DraftVertex, util::draft_app_type::DraftAppType,
};
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::{include_wgsl, util::DeviceExt};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub struct Draft {
    instance: wgpu::Instance,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    surface_configuration: Option<wgpu::SurfaceConfiguration>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    shader_module: Option<wgpu::ShaderModule>,

    triangle_pipeline: Option<wgpu::RenderPipeline>,
    triangle_vertex_buffer: Option<wgpu::Buffer>,
    triangle_index_buffer: Option<wgpu::Buffer>,
    triangle_vertex_count: u32,
    triangle_index_count: u32,

    width: u32,
    height: u32,
}

impl Draft {
    pub fn new(app_type: DraftAppType) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
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
                triangle_vertex_buffer: None,
                triangle_index_buffer: None,
                triangle_vertex_count: 0,
                triangle_index_count: 0,
                width: 600,
                height: 400,
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
                    triangle_vertex_buffer: None,
                    triangle_index_buffer: None,
                    triangle_vertex_count: 0,
                    triangle_index_count: 0,
                    width: 600,
                    height: 400,
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
        self.device = Some(device);
        self.queue = Some(queue);
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
        self.shader_module = Some(
            self.device
                .as_ref()
                .unwrap()
                .create_shader_module(include_wgsl!("shader/shader.wgsl")),
        );
        self.create_triangle_pipeline();
        self.triangle_vertex_buffer = Some(self.device.as_ref().unwrap().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Triangle Vertex Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));
        self.triangle_index_buffer = Some(self.device.as_ref().unwrap().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Triangle Index Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::INDEX,
            },
        ));
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
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(self.triangle_pipeline.as_ref().unwrap());
            render_pass
                .set_vertex_buffer(0, self.triangle_vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(
                self.triangle_index_buffer.as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..self.triangle_index_count, 0, 0..1);
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
                    bind_group_layouts: &[],
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
                    buffers: &[DraftModelVertex::desc()],
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
        self.surface_configuration.as_mut().unwrap().width = width;
        self.surface_configuration.as_mut().unwrap().height = height;
        self.surface.as_mut().unwrap().configure(
            self.device.as_ref().unwrap(),
            self.surface_configuration.as_ref().unwrap(),
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
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width != 0 && size.height != 0 {
                    self.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
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
