use wasm_bindgen::prelude::*;
mod app;
mod part1and2;
mod types;
mod controls;
mod camera;

#[wasm_bindgen]
pub fn init() {
    crate::app::run();
}
