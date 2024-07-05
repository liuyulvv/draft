mod kernel;

use kernel::{draft::Draft, util::draft_app_type::DraftAppType};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event_loop::{ControlFlow, EventLoop};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    let file_appender = tracing_appender::rolling::hourly("./", "draft.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    let event_loop = EventLoop::new();
    match event_loop {
        Ok(event_loop) => {
            event_loop.set_control_flow(ControlFlow::Poll);
            #[allow(unused_mut)]
            let mut app = Draft::new(DraftAppType::Desktop);
            let _ = event_loop.run_app(&mut app);
        }
        Err(e) => log::error!("Error creating event loop: {:?}", e),
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(log::Level::Debug);

    let event_loop = EventLoop::new();
    match event_loop {
        Ok(event_loop) => {
            event_loop.set_control_flow(ControlFlow::Poll);
            #[allow(unused_mut)]
            let mut app = Draft::new(DraftAppType::Web("main_canvas".to_string()));
            event_loop.spawn_app(app);
        }
        Err(e) => log::error!("Error creating event loop: {:?}", e),
    }
}
