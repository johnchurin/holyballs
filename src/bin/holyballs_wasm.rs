use std::sync::OnceLock;
use crossfire::mpmc;
use wasm_bindgen::prelude::wasm_bindgen;
use holyballs::{start_bevy, ExternalConsumer, ExternalMessage, ExternalProducer};

static EXTERNAL_PRODUCER: OnceLock<ExternalProducer> = OnceLock::new();

pub fn main() {
    let (tx, rx) = mpmc::bounded_async::<ExternalMessage>(3);
    // We keep the producer here for wasm calls from javascript
    EXTERNAL_PRODUCER.get_or_init(|| {
        ExternalProducer::new(tx.clone())
    });
    // The receiver is inside Bevy as a resource
    let external_consumer = ExternalConsumer::new(rx);
    start_bevy(external_consumer);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/site/js/export.js")]
extern "C" {
    fn game_ended();
    fn console_message(msg: &str);
}

// Callback from Javascript
#[wasm_bindgen]
pub fn sound(onoff: String) {
    let p = EXTERNAL_PRODUCER.get();
    if p.is_some() {
        let external_producer = p.unwrap();
        external_producer.send(ExternalMessage::new(String::from("sound"), Some(onoff)));
    }
}
#[wasm_bindgen]
pub fn load(json: String) {
    let p = EXTERNAL_PRODUCER.get();
    if p.is_some() {
        let external_producer = p.unwrap();
        external_producer.send(ExternalMessage::new(String::from("load"), Some(json)));
    }
}
#[wasm_bindgen]
pub fn play() {
    let p = EXTERNAL_PRODUCER.get();
    if p.is_some() {
        let external_producer = p.unwrap();
        external_producer.send(ExternalMessage::new(String::from("play"), None));
    }
}

#[wasm_bindgen]
pub fn end_play() {
    let p = EXTERNAL_PRODUCER.get();
    if p.is_some() {
        let external_producer = p.unwrap();
        external_producer.send(ExternalMessage::new(String::from("load"), None));
    }
}
