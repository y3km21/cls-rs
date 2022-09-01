pub mod colorset;
pub mod utils;

#[cfg(feature = "web")]
mod wasm {
    pub use ::wasm_bindgen::prelude::*;
    pub use js_sys::Uint8Array;
    pub use serde_wasm_bindgen;

    #[cfg(feature = "web")]
    #[wasm_bindgen]
    pub fn logging_init() {
        super::utils::set_panic_hook();
        wasm_logger::init(wasm_logger::Config::default());
    }
}
