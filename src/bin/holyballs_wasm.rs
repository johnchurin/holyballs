use std::sync::OnceLock;
use crossfire::{mpmc, MAsyncTx};
use crossfire::mpmc::Array;
use wasm_bindgen::prelude::wasm_bindgen;
use holyballs::{start_bevy, ExternalConsumer, ExternalMessage, ExternalProducer, ExternalReply};

static EXTERNAL_PRODUCER: OnceLock<ExternalProducer> = OnceLock::new();
static TX: OnceLock<MAsyncTx<Array<ExternalMessage>>> = OnceLock::new();

pub fn main() {
    let (tx, rx) = mpmc::bounded_async::<ExternalMessage>(3);
    // We keep the producer here for wasm calls from javascript
    EXTERNAL_PRODUCER.get_or_init(|| {
        ExternalProducer::new(tx.clone())
    });
    TX.get_or_init(|| {
        tx.clone()
    });

    // The receiver is inside Bevy as a resource
    let external_consumer = ExternalConsumer::new(rx);
    let external_reply = ExternalReply::new(reply_handler);
    start_bevy(external_consumer, external_reply);
}

#[wasm_bindgen(module = "/site/js/export.js")]
extern "C" {
    fn game_ended();
    fn console_message(msg: &str);
}
// Callback from game
fn reply_handler(message: ExternalMessage) {
    let tx_handle = TX.get();
    if tx_handle.is_some() {
        let _tx = tx_handle.unwrap();
        match message.action.as_str() {
            "game_ended" => {
                game_ended();
            }
            _ => {
                println!("Invalid reply message from game: {:?}", message.action);
            }
        }
    }
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
        external_producer.send(ExternalMessage::new(String::from("end_play"), None));
    }
}
