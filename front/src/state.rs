use crate::response::{Ligne, Position, Positions};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::*;

#[wasm_bindgen]
pub struct State {
    lignes: Vec<Ligne>,
    positions: Vec<Position>,
}

#[wasm_bindgen]
impl State {
    #[wasm_bindgen(constructor)]
    pub fn new() -> State {
        State {
            lignes: Vec::new(),
            positions: Vec::new(),
        }
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
