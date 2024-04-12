use sauron::*;

use app::App;

mod app;

fn main() {
    let app = App{};
    println!("{}", app.view().render_to_string());
}
