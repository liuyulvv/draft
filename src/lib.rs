mod app;
mod state;

use crate::app::App;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event_loop::{ControlFlow, EventLoop};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    #[allow(unused_mut)]
    let mut app = App::new().await;

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    cfg_if::cfg_if! {
        if #[cfg(target_arch="wasm32")] {
            use winit::platform::web::EventLoopExtWebSys;
            let _= event_loop.spawn_app(app);
        } else {
            let _ = event_loop.run_app(&mut app);
        }
    }
}
