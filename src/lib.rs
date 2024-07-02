mod app;
mod state;
mod util;

use crate::app::App;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(not(target_arch = "wasm32"))]
pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    match event_loop {
        Ok(event_loop) => {
            event_loop.set_control_flow(ControlFlow::Poll);
            #[allow(unused_mut)]
            let mut app = App::new().await;
            let _ = event_loop.run_app(&mut app);
        }
        Err(e) => log::error!("Error creating event loop: {:?}", e),
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct DraftApp {
    #[wasm_bindgen(skip)]
    pub app: App,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl DraftApp {
    pub async fn new() -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        let _ = console_log::init_with_level(log::Level::Debug);
        #[allow(unused_mut)]
        let mut app = App::new().await;
        Self { app }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.app.resize(util::size::Size { width, height });
    }

    pub fn draw(&mut self) {
        self.app.draw();
    }
}
