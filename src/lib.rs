mod components;
pub mod main_loop;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run(width: u32, height: u32) {
    init_logger();
    main_loop::main_loop(width, height).await;
}

fn init_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

        } else {
            env_logger::init();
        }
    }
}