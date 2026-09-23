//! The browser input protocol and JavaScript bridge.
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = MossBridge, js_name = takeInput)]
    pub(super) fn take_input() -> String;
    #[wasm_bindgen(js_namespace = MossBridge)]
    pub(super) fn render(snapshot: &str);
    #[wasm_bindgen(js_namespace = MossBridge)]
    pub(super) fn fail(message: &str);
}

#[derive(Deserialize)]
pub(super) struct Input {
    pub(super) now_ms: f64,
    pub(super) hidden: bool,
    pub(super) hidden_epoch: u32,
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) actions: Vec<Action>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum Action {
    TogglePlay,
    Step,
    Reset,
    Fit,
    Select { id: String },
    Pan { dx: f32, dy: f32 },
    Zoom { x: f32, y: f32, factor: f32 },
    Click { x: f32, y: f32 },
}
