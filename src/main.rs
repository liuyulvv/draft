mod app;
mod state;

use crate::app::App;

fn main() {
    let mut app = App::default();
    app.run();
}
