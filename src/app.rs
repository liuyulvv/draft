use crate::{state, util};
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    state: Option<state::State>,
}

impl App {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        Self {
            window: None,
            state: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new() -> Self {
        let window = web_sys::window().expect("No global `window` exists");
        let document = window.document().expect("Should have a document on window");
        let canvas = document.get_element_by_id("main_canvas").unwrap();
        let canvas = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Show have a canvas");
        Self {
            window: None,
            state: Some(state::State::new(canvas).await),
        }
    }

    pub fn resize(&mut self, size: util::size::Size<u32>) {
        if let Some(state) = &mut self.state {
            state.resize(size);
            self.window.as_ref().unwrap().request_redraw();
        }
    }

    pub fn draw(&mut self) {
        if let Some(state) = &mut self.state {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => {
                    state.resize(state.size());
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                }
            }
            self.window.as_ref().unwrap().request_redraw();
        }
    }
}

impl ApplicationHandler for App {
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
            let mut _state = pollster::block_on(state::State::new(window.clone()));
            self.state = Some(_state);
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
                    self.resize(util::size::Size {
                        width: size.width,
                        height: size.height,
                    });
                }
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            _ => {}
        }
    }
}
