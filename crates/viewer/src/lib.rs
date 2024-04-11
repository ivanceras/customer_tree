#![deny(warnings)]

use wasm_bindgen::prelude::*;
use app::App;
use error::Error;
use sauron::Program;

mod customer;
mod app;
mod error;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Info).unwrap();
    console_error_panic_hook::set_once();
    let app_container = sauron::document()
        .get_element_by_id("app_container")
        .expect("must have the app_container in index.html");
    Program::replace_mount(App::new(), &app_container);
}

