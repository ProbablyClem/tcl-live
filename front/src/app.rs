use crate::panic;
use crate::render;
use crate::response::{Ligne, Position, Positions};
use crate::webgl::WebGlRenderer;
use crate::webgl::init_webgl_context;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::*;
#[wasm_bindgen]
pub struct App {
    lignes: Vec<Ligne>,
    positions: Vec<Position>,
    renderer: WebGlRenderer,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            lignes: Vec::new(),
            positions: Vec::new(),
            renderer: WebGlRenderer::new(),
        }
    }

    pub fn start(&self) {
        panic::set_panic_hook();
    }

    pub fn render(&self) {
        console::log_1(&"Render".into());
        self.renderer.render();
    }

    pub fn set_lignes(&mut self, data: JsValue) -> Result<(), JsValue> {
        let lignes: Vec<Ligne> =
            serde_wasm_bindgen::from_value(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.lignes = lignes;
        console::log_1(&format!("set lignes : {}", self.lignes.len()).into());
        Ok(())
    }

    pub fn set_positions(&mut self, data: JsValue) -> Result<(), JsValue> {
        let positions: Positions =
            serde_wasm_bindgen::from_value(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.positions = positions.positions;
        console::log_1(&format!("set positions : {}", self.positions.len()).into());
        Ok(())
    }
}
