use crate::state;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
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
    pub async fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch="wasm32")] {
                let window = web_sys::window().expect("No global `window` exists");
                let document = window.document().expect("Should have a document on window");
                let canvas = document.get_element_by_id("main_canvas").unwrap();
                let canvas = canvas
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .expect("Show have a canvas");
                Self {
                    window: None,
                    state: Some(state::State::create_from_canvas(canvas).await),
                }
            } else {
                Self {
                    window: None,
                    state: None,
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            cfg_if::cfg_if! {
                if #[cfg(target_arch="wasm32")] {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    let window = web_sys::window().expect("No global `window` exists");
                    let document = window.document().expect("Should have a document on window");
                    let canvas = document.get_element_by_id("main_canvas").unwrap();
                    let canvas = canvas
                        .dyn_into::<web_sys::HtmlCanvasElement>()
                        .expect("Show have a canvas");
                    let canvas = Some(canvas);
                    let win_attr = Window::default_attributes().with_canvas(canvas);
                } else {
                    let win_attr = Window::default_attributes().with_title("Draft");
                }
            }
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("Create window err."),
            );
            self.window = Some(window.clone());
            cfg_if::cfg_if! {
                if #[cfg(not(target_arch="wasm32"))] {
                    let mut _state = pollster::block_on(state::State::create_from_window(window.clone()));
                    self.state = Some(_state);
                }
            }
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
                if let Some(state) = &mut self.state {
                    if size.width == 0 || size.height == 0 {
                        // window minimized
                    } else {
                        state.resize(size);
                        self.window.as_ref().unwrap().request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
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
            _ => {}
        }
    }
}
