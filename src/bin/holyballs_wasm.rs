use std::sync::OnceLock;
use crossfire::{mpmc, MAsyncTx};
use crossfire::mpmc::Array;
use wasm_bindgen::prelude::wasm_bindgen;
use holyballs::{start_bevy, ExternalConsumer, ExternalMessage, ExternalProducer, ExternalReply};

static EXTERNAL_PRODUCER: OnceLock<ExternalProducer> = OnceLock::new();
static TX: OnceLock<MAsyncTx<Array<ExternalMessage>>> = OnceLock::new();

pub fn main() {
    let (tx, rx) = mpmc::bounded_async::<ExternalMessage>(30);
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
unsafe extern "C" {
    fn game_ended();
    fn latest_score(score: String);
    fn console_message(msg: &str);
    fn play_sound(file: String);
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
            "update_score" => {
                latest_score(message.payload.unwrap());
            }
            "play_sound" => {
                if message.payload.is_some() {
                    play_sound(message.payload.unwrap());
                }
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
pub fn play(json: String, game_name: String) {
    let p = EXTERNAL_PRODUCER.get();
    if p.is_some() {
        let external_producer = p.unwrap();
        external_producer.send(ExternalMessage::new(String::from("load"), Some(json)));
        external_producer.send(ExternalMessage::new(String::from("play"), Some(game_name)));
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
